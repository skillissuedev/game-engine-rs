#/bin/bash

unlink target/debug/assets
unlink target/release/assets
cp -r ../baldej_assets target/release/
mv target/release/baldej_assets target/release/assets 
cp -r ../baldej_assets target/debug/
mv target/debug/baldej_assets target/debug/assets 
