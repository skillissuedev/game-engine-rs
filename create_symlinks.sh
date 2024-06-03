#!/bin/bash

cd target/release/
ln -s ../../../game_assets assets
cd ../debug/
ln -s ../../../game_assets assets
