#!/bin/bash
cd ../
cargo bundle --release
cd ./deploy_scripts/

codesign -s "University of Wisconsin-Madison" --deep -f -v -o runtime --entitlements ./entitlements.plist ../target/release/bundle/osx/Foray.app

zip -r ../target/release/bundle/osx/Foray.zip  ../target/release/bundle/osx/Foray.app/

# credentials must already be stored via:
# xcrun notarytool store-credentials "notarytool-password" \
#                         --apple-id "johnnyc1423@gmail.com" \
#                         --team-id C658W77DY8 \
#                         --password <app specific password created from App-Specific Passwords" https://account.apple.com/account/manage>
xcrun notarytool submit ../target/release/bundle/osx/Foray.zip --keychain-profile "notarytool-password" --wait

# then create a release with something like this:
# gh release create v0.1.2 --notes "application is now notarized" ./Foray.zip 
# todo: change file name to have version/architecture

# then update uw-mrtud/homebrew-mrtud/ to use the newest version/release

