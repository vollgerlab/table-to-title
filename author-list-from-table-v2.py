#!/usr/bin/env python
import argparse
import os
import sys
import pandas as pd
import numpy as np

# global var for inputs for new format
AFF1 = "Affilation 1"
AFF2 = "Affilation 2"
AFF3 = "Affiliation 3"
FIRST = "First"
MIDDLE = "M"
LAST = "Last Name"
EMAIL = "Email"


def add_symbol(x, value="†"):
    if x == "" or x is None or pd.isna(x):
        return ""
    return value


def add_first(x):
    return add_symbol(x, value="*")


def add_corresponding(x):
    return add_symbol(x, value="†")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="", formatter_class=argparse.ArgumentDefaultsHelpFormatter
    )
    parser.add_argument("infile", help="positional input")
    parser.add_argument("-s", "--string", help="string option")
    parser.add_argument("-n", "--number", help="numeric option", type=int, default=5)
    parser.add_argument(
        "-l", "--list", nargs="*", help="list with zero or more entries"
    )
    parser.add_argument("-l2", "--list2", nargs="+", help="list one or more entries")
    parser.add_argument(
        "-d", help="store args.d as true if -d", action="store_true", default=False
    )
    args = parser.parse_args()
    df = pd.read_csv(args.infile, sep="\t")
    df[MIDDLE] = df[MIDDLE].fillna("")

    # Combine affiliation columns into single list
    aff_cols = [AFF1, AFF2, AFF3]
    affiliations = []
    for idx, row in df.iterrows():
        affs = []
        for col in aff_cols:
            if col in df.columns and pd.notna(row[col]) and row[col].strip() != "":
                affs.append(row[col].strip())
        affiliations.append("; ".join(affs) if affs else "")

    df["AFF_COMBINED"] = affiliations

    aff_nums = {}
    for aff_string in df["AFF_COMBINED"]:
        if aff_string:
            affs = [x.strip() for x in aff_string.split(";")]
            for aff in affs:
                if aff and aff not in aff_nums:
                    aff_nums[aff] = len(aff_nums) + 1

    df["nums"] = df["AFF_COMBINED"].apply(
        lambda x: ",".join(
            [str(aff_nums[aff.strip()]) for aff in x.split(";") if aff.strip()]
        ) if x else ""
    )

    df["Name"] = (
        (
            df[FIRST].fillna("")
            + " "
            + df[MIDDLE]
            + " "
            + df[LAST].fillna("")
            + df["nums"]
        )
        .str.replace("  ", " ")
        .str.strip()
    )
    # print(df)

    # Filter out rows with empty names
    df = df[df["Name"].str.strip() != ""]

    print(", ".join(df.Name))
    print("")
    for aff, num in aff_nums.items():
        print(f"{num}. {aff.strip()}")
