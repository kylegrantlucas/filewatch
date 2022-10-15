#!/bin/bash

# This script generates test data for the test suite.
# Get the path for test data from cli args
TESTDATA_PATH=$1

# Clear old test data
rm -rf $TESTDATA_PATH

# Create the test data directory
mkdir -p $TESTDATA_PATH

# Create 10 test dirs
for i in {1..10}; do
    mkdir -p $TESTDATA_PATH/test_$i
done

# Create 100 test files in each test dir with txt extension
for i in {1..10}; do
    for j in {1..100}; do
        file=$TESTDATA_PATH/test_$i/testfile_$j.txt
        touch $file
        echo "This is test file $file in test dir $i" > $file
    done
done

# Create 100 test files in each test dir with md extension
for i in {1..10}; do
    for j in {1..100}; do
        file=$TESTDATA_PATH/test_$i/testfile_$j.md
        touch $file
        echo "This is test file $file in test dir $i" > $file
    done
done

# Create moved dir
mkdir -p $TESTDATA_PATH/moved

# Create backup dir
mkdir -p $TESTDATA_PATH/backup
