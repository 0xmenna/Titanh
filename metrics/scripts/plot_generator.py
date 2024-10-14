import pandas as pd # type: ignore
import matplotlib.pyplot as plt # type: ignore
import os

def main():
    put_metrics_graph()
    get_metrics_graph()

def put_metrics_graph():
    output_dir = "./plot/put/"
    
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    dataframe: pd.DataFrame = pd.read_csv(f"./../data/put_metrics.csv")

    dataframe = dataframe.rename(columns={
        "size": "Size [MB]",
        "consistency_level": "Consistency Level",
        "avg_elapsed_time_ms": "Avg Elapsed Time [ms]"
    })

    dataframe["Size [MB]"] = dataframe["Size [MB]"] / (1024 * 1024)

    colors = ['blue', 'green', 'red']

    for i, level in enumerate(dataframe["Consistency Level"].unique()):
        level_data = dataframe[dataframe["Consistency Level"] == level]
    
        plt.figure()
        plt.plot(
            level_data["Size [MB]"], 
            level_data["Avg Elapsed Time [ms]"], 
            label=f"{level}", 
            marker="o",
            color=colors[i % len(colors)]
        )

        plt.xlabel("Size [MB]")
        plt.ylabel("Avg Elapsed Time [ms]")
        plt.title(f"Elapsed Time vs Size [MB] for {level} Consistency Level")
        
        plt.legend(title="Consistency Level", fontsize='small')  
        plt.grid(True)

        plt.savefig(f"./plot/put/put_metrics_{level}_graph.jpg", transparent=True)
        
def get_metrics_graph():
    output_dir = "./plot/get/"
    
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    dataframe: pd.DataFrame = pd.read_csv(f"./../data/get_metrics.csv")

    dataframe = dataframe.rename(columns={
        "size": "Size [MB]",
        "consistency_level": "Consistency Level",
        "avg_elapsed_time_ms": "Avg Elapsed Time [ms]"
    })

    dataframe["Size [MB]"] = dataframe["Size [MB]"] / (1024 * 1024)

    avg_dataframe = dataframe.groupby(["Consistency Level", "Size [MB]"]).mean().reset_index()

    avg_dataframe = avg_dataframe.sort_values(by="Size [MB]")

    colors = ['blue', 'green']

    for i, level in enumerate(avg_dataframe["Consistency Level"].unique()):
        level_data = avg_dataframe[avg_dataframe["Consistency Level"] == level]

        plt.figure()
        plt.plot(
            level_data["Size [MB]"], 
            level_data["Avg Elapsed Time [ms]"], 
            label=f"{level}", 
            marker="o",
            color=colors[i % len(colors)]
        )

        plt.xlabel("Size [MB]")
        plt.ylabel("Avg Elapsed Time [ms]")
        plt.title(f"Elapsed Time vs Size [MB] for {level} Consistency Level (Get)")
        
        plt.legend(title="Consistency Level", fontsize='small')  
        plt.grid(True)

        plt.savefig(f"./plot/get/get_metrics_{level}_graph.jpg", transparent=True)


if __name__ == "__main__":
    main()