import matplotlib.pyplot as plt
from typing import Dict, List, Tuple
from collections import defaultdict
import pandas as pd
from argparse import ArgumentParser

LINE_STYLE = ["s-", "^-", "o--", ".:"]
LINE_COLOR = ["k", "#d95f02", "#1b9e77", "#7570b3"]

SCALING = 100000
HEIGHT = 3
WIDTH = 5

IMAGES_DIR = "img"
LOG_DIR = "log"
CSV_DIR = "csv"


def make_csv(filename: str, out_name: str):
    with open(f"{LOG_DIR}/{filename}") as in_file:
        in_lines = in_file.readlines()
        with open(f"{CSV_DIR}/{out_name}", "w") as out_file:
            valid = False
            for line in in_lines:
                if line.endswith(" ... ok\n"):
                    return
                elif line.endswith(" has been running for over 60 seconds\n"):
                    continue
                if valid:
                    out_file.write(line)
                if line == "running 1 test\n":
                    valid = True


# For constant # elements, how does # cores change things?
def vary_cores(df, df_o, df_rw, ax, buf_size):
    ax.grid(linestyle="dotted")
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    size_mask = df["buf_size"] == buf_size
    sub_df = df[size_mask]
    ax.plot(
        sub_df["num_cores"],
        sub_df["stm_ops_per_sec"] / SCALING,
        LINE_STYLE[1],
        color=LINE_COLOR[1],
        label=f"TORTIS",
    )
    ax.plot(
        sub_df["num_cores"],
        sub_df["swym_ops_per_sec"] / SCALING,
        LINE_STYLE[2],
        color=LINE_COLOR[2],
        label=f"TL2",
    )
    size_mask_o = df_o["buf_size"] == buf_size
    sub_df_o = df_o[size_mask_o]
    ax.plot(
        sub_df_o["num_cores"],
        sub_df_o["swym_o_ops_per_sec"] / SCALING,
        LINE_STYLE[3],
        color=LINE_COLOR[3],
        label=f"TL2-O",
    )
    size_mask_rw = df_rw["buf_size"] == buf_size
    sub_df_rw = df_rw[size_mask_rw]
    ax.plot(
        sub_df_rw["num_cores"],
        sub_df_rw["stm_ops_per_sec"] / SCALING,
        LINE_STYLE[0],
        color=LINE_COLOR[0],
        label=f"TORTIS R/W",
    )
    ax.set_xlabel("Number of cores")
    ax.set_ylabel("Throughput (100K TX/s)")
    ax.set_yscale("log")
    ax.set_xticks(
        [1, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36]
    )
    ax.legend()


# For constant # cores, how does # elements accessed change things?
def vary_percent_access(df, df_o, df_rw, ax, buf_size: int, num_cores: int):
    ax.grid(linestyle="dotted")
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    size_mask = df["buf_size"] == buf_size
    core_mask = df["num_cores"] == num_cores
    sub_df = df[size_mask & core_mask]
    ax.plot(
        sub_df["percent_accessed"],
        sub_df["stm_ops_per_sec"] / SCALING,
        LINE_STYLE[1],
        color=LINE_COLOR[1],
        label=f"TORTIS",
    )
    ax.plot(
        sub_df["percent_accessed"],
        sub_df["swym_ops_per_sec"] / SCALING,
        LINE_STYLE[2],
        color=LINE_COLOR[2],
        label=f"TL2",
    )
    size_mask_o = df_o["buf_size"] == buf_size
    core_mask_o = df_o["num_cores"] == num_cores
    sub_df_o = df_o[size_mask_o & core_mask_o]
    ax.plot(
        sub_df_o["percent_accessed"],
        sub_df_o["swym_o_ops_per_sec"] / SCALING,
        LINE_STYLE[3],
        color=LINE_COLOR[3],
        label=f"TL2-O",
    )
    size_mask_rw = df_rw["buf_size"] == buf_size
    core_mask_rw = df_rw["num_cores"] == num_cores
    sub_df_rw = df_rw[size_mask_rw & core_mask_rw]
    ax.plot(
        sub_df_rw["percent_accessed"],
        sub_df_rw["stm_ops_per_sec"] / SCALING,
        LINE_STYLE[0],
        color=LINE_COLOR[0],
        label=f"TORTIS R/W",
    )
    ax.set_xlabel("Percent of elements accessed (%)")
    ax.set_ylabel("Throughput (100K TX/s)")
    ax.set_yscale("log")
    ax.set_xticks([0.10, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],)
    ax.set_xticklabels(["10", "20", "30", "40", "50", "60", "70", "80", "90", "100"])
    ax.legend()


# For constant # cores, how does # elements change things?
def vary_write_load(df, df_o, df_rw, ax, buf_size: int, num_cores: int):
    ax.grid(linestyle="dotted")
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    size_mask = df["buf_size"] == buf_size
    core_mask = df["num_cores"] == num_cores
    sub_df = df[size_mask & core_mask]
    ax.plot(
        sub_df["percent_writes"],
        sub_df["stm_ops_per_sec"] / SCALING,
        LINE_STYLE[1],
        color=LINE_COLOR[1],
        label=f"TORTIS",
    )
    ax.plot(
        sub_df["percent_writes"],
        sub_df["swym_ops_per_sec"] / SCALING,
        LINE_STYLE[2],
        color=LINE_COLOR[2],
        label=f"TL2",
    )
    size_mask_o = df_o["buf_size"] == buf_size
    core_mask_o = df_o["num_cores"] == num_cores
    sub_df_o = df_o[size_mask_o & core_mask_o]
    ax.plot(
        sub_df_o["percent_writes"],
        sub_df_o["swym_o_ops_per_sec"] / SCALING,
        LINE_STYLE[3],
        color=LINE_COLOR[3],
        label=f"TL2-O",
    )
    size_mask_rw = df_rw["buf_size"] == buf_size
    core_mask_rw = df_rw["num_cores"] == num_cores
    sub_df_rw = df_rw[size_mask_rw & core_mask_rw]
    ax.plot(
        sub_df_rw["percent_writes"],
        sub_df_rw["stm_ops_per_sec"] / SCALING,
        LINE_STYLE[0],
        color=LINE_COLOR[0],
        label=f"TORTIS R/W",
    )
    ax.set_xlabel("Write percentage (%)")
    ax.set_ylabel("Throughput (100K TX/s)")
    ax.set_yscale("log")
    ax.set_xticks([0.0, 0.10, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],)
    ax.set_xticklabels(
        ["0", "10", "20", "30", "40", "50", "60", "70", "80", "90", "100"]
    )
    ax.legend()


def main(filename: str, buf_size: int, num_cores: int):
    fig = plt.figure()
    ax = fig.add_subplot(1, 1, 1)
    prefix = filename.split(".")[0]
    make_csv(filename, f"{prefix}.csv")
    df = pd.read_csv(f"{CSV_DIR}/{prefix}.csv")
    print(df.columns)
    make_csv(f"{prefix}_o.log", f"{prefix}_o.csv")
    df_o = pd.read_csv(f"{CSV_DIR}/{prefix}_o.csv")
    make_csv(f"{prefix}_rw.log", f"{prefix}_rw.csv")
    df_rw = pd.read_csv(f"{CSV_DIR}/{prefix}_rw.csv")

    if filename.startswith("out1"):
        vary_cores(df, df_o, df_rw, ax, buf_size)
    elif filename.startswith("out2"):
        vary_percent_access(df, df_o, df_rw, ax, buf_size, num_cores)
    elif filename.startswith("out3"):
        vary_write_load(df, df_o, df_rw, ax, buf_size, num_cores)
    fig.set_size_inches(WIDTH, HEIGHT)
    plt.tight_layout()
    plt.savefig(f"{IMAGES_DIR}/{prefix}_{buf_size}_{num_cores}_cores.png")
    # plt.show()


if __name__ == "__main__":
    """
    parser = ArgumentParser()
    parser.add_argument("filename", type=str, help="the name of the log file to read")
    parser.add_argument("buf_size", type=int, help="the size of buffer to use")
    parser.add_argument("num_cores", type=int, help="the number of cores")
    args = parser.parse_args()
    main(args.filename, args.buf_size, args.num_cores)
    """
    main("out1.log", 64, 8)
    main("out1.log", 128, 8)
    main("out1.log", 256, 8)
    main("out1.log", 64, 16)
    main("out1.log", 128, 16)
    main("out1.log", 256, 16)

    main("out2.log", 64, 8)
    main("out2.log", 128, 8)
    main("out2.log", 256, 8)
    main("out2.log", 64, 16)
    main("out2.log", 128, 16)
    main("out2.log", 256, 16)

    main("out3.log", 64, 8)
    main("out3.log", 128, 8)
    main("out3.log", 256, 8)
    main("out3.log", 64, 16)
    main("out3.log", 128, 16)
    main("out3.log", 256, 16)
