use anyhow::Result;
use sp_core::H256;
use titan_api::{
    titanh::{
        self,
        capsules::events::{CapsuleItemsDeleted, CapsuleStartedDestroying},
    },
    TitanhApi, TitanhApiBuilder,
};

pub struct GarbageCollectorConsumer {
    /// The seed phrase of the garbage collector node
    collector_seed: String,
    /// The rpc url of the substrate node
    rpc_url: String,
    /// Key range of the garbage collector
    key_range: Option<(H256, H256)>,
}

impl GarbageCollectorConsumer {
    pub fn new(collector_seed: String, rpc_url: String, key_range: Option<(H256, H256)>) -> Self {
        GarbageCollectorConsumer {
            collector_seed,
            rpc_url,
            key_range,
        }
    }

    pub async fn start(self) -> Result<()> {
        let api = TitanhApiBuilder::rpc(&self.rpc_url)
            .seed(&self.collector_seed)
            .build()
            .await?;

        let mut blocks_sub = api.substrate_api.blocks().subscribe_finalized().await?;

        while let Some(block) = blocks_sub.next().await {
            let block = block?;
            log::info!("Processing block: {:?}", block.number());
            let events = block.events().await?;

            for event in events.iter() {
                let event = event?;

                let maybe_destroying_event = event.as_event::<CapsuleStartedDestroying>()?;

                if let Some(destroying_event) = maybe_destroying_event {
                    log::info!("Recieved destroying event: {:?}", destroying_event);

                    let key = destroying_event.capsule_id;
                    if self.is_key_in_range(&key) {
                        log::info!("Destroying capsule with key: {:?}", key);
                        destroy_capsule(&api, key).await?;
                    }
                }
            }
        }

        Ok(())
    }

    fn is_key_in_range(&self, key: &H256) -> bool {
        if let Some((start, end)) = self.key_range {
            *key >= start && *key <= end
        } else {
            // if no key range is set, the key is always in range
            true
        }
    }
}

pub async fn destroy_capsule(api: &TitanhApi, capsule_id: H256) -> Result<()> {
    let capsules_tx = titanh::tx().capsules();

    let mut garbage_phase = GarbageCollectionPhase::OwnershipApprovals;

    loop {
        log::info!("Garbage collection phase: {:?}", garbage_phase);

        match garbage_phase {
            GarbageCollectionPhase::OwnershipApprovals => {
                let ownership_deletion =
                    capsules_tx.destroy_capsule_ownership_approvals(capsule_id);
                let res = api
                    .sign_and_submit_wait_finalized(&ownership_deletion)
                    .await;

                if let Ok(events) = res {
                    let completition_event = events.find_first::<CapsuleItemsDeleted>()?.unwrap();
                    let has_completed = completition_event.removal_completion;

                    if has_completed {
                        garbage_phase = GarbageCollectionPhase::Followers;
                    }
                } else {
                    // Someone else has already garbage collected
                    garbage_phase = GarbageCollectionPhase::Followers;
                }
            }
            GarbageCollectionPhase::Followers => {
                let followers_deletion = capsules_tx.destroy_capsule_followers(capsule_id);
                let res = api
                    .sign_and_submit_wait_finalized(&followers_deletion)
                    .await;

                if let Ok(events) = res {
                    let completition_event = events.find_first::<CapsuleItemsDeleted>()?.unwrap();
                    let has_completed = completition_event.removal_completion;

                    if has_completed {
                        garbage_phase = GarbageCollectionPhase::ContainerKeys;
                    }
                } else {
                    // Someone else has already garbage collected
                    garbage_phase = GarbageCollectionPhase::ContainerKeys;
                }
            }
            GarbageCollectionPhase::ContainerKeys => {
                let container_keys_deletion =
                    capsules_tx.destroy_capsule_container_keys(capsule_id);
                let res = api
                    .sign_and_submit_wait_finalized(&container_keys_deletion)
                    .await;

                if let Ok(events) = res {
                    let completition_event = events.find_first::<CapsuleItemsDeleted>()?.unwrap();
                    let has_completed = completition_event.removal_completion;

                    if has_completed {
                        garbage_phase = GarbageCollectionPhase::FinishDestroy;
                    }
                } else {
                    // Someone else has already garbage collected
                    garbage_phase = GarbageCollectionPhase::FinishDestroy;
                }
            }
            GarbageCollectionPhase::FinishDestroy => {
                let finish_destroy = capsules_tx.finish_destroy_capsule(capsule_id);
                let _ = api.sign_and_submit_wait_finalized(&finish_destroy).await;

                garbage_phase = GarbageCollectionPhase::Exit;
            }
            GarbageCollectionPhase::Exit => break,
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GarbageCollectionPhase {
    OwnershipApprovals,
    Followers,
    ContainerKeys,
    FinishDestroy,
    Exit,
}
