use codec::Encode;
use sp_core::H256;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use titan_api::{CapsulesApi, TitanhApi};

pub const CHAIN_ENDPOINT: &str = "ws://127.0.0.1:9944";
pub const IPFS_ENDPOINT: &str = "http://127.0.0.1:5001";

const RING_SIZE: usize = 6;

const PINNING_RIG: [&str; RING_SIZE] = [
    "0x1647b73ff0310ecfb350d26c8ea9353142b83ec3cb2ab9d08989209612833939",
    "0x460ef096e9d9e30bedaa7b2bac9b695869f1cc19adb2035e91f25a041d851578",
    "0x9e18440d18acfcca2530f2af98aac10aa6e8f2e7d81881432b5763dde65e78b4",
    "0xdb55dd11bc7b7e53e1ffdc984275ce5490f0e1998456e4dda268408098385033",
    "0xe04137c83bfc446db46725d3680a8888cbe6157576eec5197f8493be79b933e4",
    "0xf9ce5baddb499ce6b02c4d9876a2e38e773c3ac22eb28b6d07d60461da0af423",
];
const REP_FACTOR: u32 = 2;

pub const SEED_PRHASE: &str =
    "bread february program comic unveil clock output oblige jewel tell reunion hammer";

pub const APP: u32 = 1;

pub const NUM_CAPSULES: usize = 30;

pub const LEAVE_NODE_IDX: usize = 3;

pub const KEYS_DIR: &str = "ring";
pub const KEYS_LEAVE_DIR: &str = "ring-after-leave";

pub struct MockRing {
    rep_factor: u32,
    ring: Vec<H256>,
}

impl MockRing {
    pub fn mock() -> Self {
        let ring: Vec<H256> = PINNING_RIG
            .into_iter()
            .map(|key| {
                let key = key[2..].as_bytes();
                let key = hex::decode(key).unwrap();
                H256::from_slice(&key)
            })
            .collect();

        Self {
            rep_factor: REP_FACTOR,
            ring,
        }
    }

    fn find_replicas(&self, key: &H256) -> Vec<H256> {
        let idx = match self.ring.binary_search(key) {
            Ok(index) => index,
            Err(index) => {
                if index == self.ring.len() {
                    0
                } else {
                    index
                }
            }
        };

        let mut replicas = Vec::new();
        let k = self.rep_factor as usize;
        let mut i: usize = 0;
        while i < k {
            let replica = self.ring[(idx + i) % self.ring.len()];
            replicas.push(replica);
            i += 1;
        }

        replicas
    }
}

pub struct MockApi<'a> {
    pub capsules: CapsulesApi<'a>,
    ring: MockRing,
    // (node, patition_num) -> keys
    assigned_keys: HashMap<(H256, u32), Vec<H256>>,
    keys: Vec<H256>,
}

impl<'a> MockApi<'a> {
    pub fn mock_from_api(api: &'a TitanhApi) -> MockApi<'a> {
        let mock_ring = MockRing::mock();
        let mut assigned_keys = HashMap::new();
        for node in mock_ring.ring.iter() {
            for idx in 0..mock_ring.rep_factor {
                assigned_keys.insert((*node, idx + 1), Vec::new());
            }
        }

        MockApi {
            capsules: api.capsules().config(IPFS_ENDPOINT, APP).unwrap(),
            ring: mock_ring,
            assigned_keys,
            keys: Vec::new(),
        }
    }

    pub fn adjust_nodes_keys(&mut self, key: &H256) {
        let replicas = self.ring.find_replicas(&key);

        for (idx, replica) in replicas.iter().enumerate() {
            let keys = self
                .assigned_keys
                .get_mut(&(*replica, idx as u32 + 1))
                .unwrap();
            keys.push(*key);
            keys.sort();
        }
    }

    pub fn assign_key_to_replicas<Id: Encode>(&mut self, id: Id) {
        let key = self.capsules.compute_capsule_id(&id, APP);
        self.keys.push(key);

        self.adjust_nodes_keys(&key);
    }

    pub fn display_assigned_keys(&self, dir_path: &str) {
        for (idx, node) in self.ring.ring.iter().enumerate() {
            // Build file path
            let file_path = format!("{}/node-{:?}.txt", dir_path, idx + 1);

            // Create and open the file
            let mut file = File::create(&file_path).expect("Unable to create file");

            // Write to file
            writeln!(file, "Pinning Node: {:?}", node).unwrap();

            for idx in 0..self.ring.rep_factor {
                writeln!(
                    file,
                    "==============================  Keys of Row {} ==============================",
                    idx + 1
                )
                .unwrap();
                let keys = self.assigned_keys.get(&(*node, idx + 1)).unwrap();
                for key in keys {
                    writeln!(file, "{:?}", key).unwrap();
                }
                writeln!(
                    file,
                    "===================================================================="
                )
                .unwrap();
            }
            writeln!(file).unwrap();
        }
    }

    pub fn display_leave_simulation(&mut self, dir_path: &str) {
        let leave_node = self.ring.ring.remove(LEAVE_NODE_IDX);

        self.assigned_keys.clear();
        for node in self.ring.ring.iter() {
            for idx in 0..self.ring.rep_factor {
                self.assigned_keys.insert((*node, idx + 1), Vec::new());
            }
        }

        let keys = self.keys.clone();
        for key in keys.iter() {
            self.adjust_nodes_keys(key);
        }

        // Write the message to a file
        let leave_info_path = format!("{}/leave_info.txt", dir_path);
        let mut info_file =
            File::create(&leave_info_path).expect("Unable to create leave_info.txt");

        writeln!(info_file, "Leaving node: {:?}", leave_node).unwrap();

        self.display_assigned_keys(dir_path);
    }
}
