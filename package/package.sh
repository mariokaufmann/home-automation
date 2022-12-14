#!/bin/bash

rm -rf ./target
rm -f "home-automation-$1.zip"

mkdir ./target
cp ../target/release/home-automation-server target/
cp ../target/release/home-automation-streamdeck-client target/

cd target/ && zip -r "../home-automation-$1.zip" *
