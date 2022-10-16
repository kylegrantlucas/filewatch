#!/bin/bash

# This script generates test data for the test suite.
# Get the path for test data from cli args
TESTDATA_PATH=$1

# Clear old test data
rm -rf "$TESTDATA_PATH"

# Create the test data directory
mkdir -p "$TESTDATA_PATH"

folder_size=$2
file_size=$3

# Create 10 test dirs
for ((i=1;i<=folder_size;i++)); do
    mkdir -p "$TESTDATA_PATH/test_$i"
done

# Create 100 test files in each test dir with txt extension

for ((i=1;i<=folder_size;i++)); do
    for ((j=1;j<=file_size;j++)); do
        touch "$TESTDATA_PATH/test_$i/testfile_$i-$j.txt" &
    done
done

# Create 100 test files in each test dir with md extension
for ((i=1;i<=folder_size;i++)); do
    for ((j=1;j<=file_size;j++)); do
        touch "$TESTDATA_PATH/test_$i/testfile_$i-$j.md" &
    done
done

wait

# Create moved dir
mkdir -p "$TESTDATA_PATH/moved"

# Create backup dir
mkdir -p "$TESTDATA_PATH/backup"

# Create backup2 dir
mkdir -p "$TESTDATA_PATH/backup2"
