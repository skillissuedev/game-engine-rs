#/bin/bash

unlink target/debug/assets
unlink target/release/assets
cp -r ../game_assets target/release/
mv target/release/game_assets target/release/assets 
cp -r ../game_assets target/debug/
mv target/debug/game_assets target/debug/assets 
