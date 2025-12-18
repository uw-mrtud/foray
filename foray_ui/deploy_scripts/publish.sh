#!/bin/bash

foray_file_zip="./Foray-$1.zip"
echo "Creating release: $1"
echo "foray file: $foray_file_zip"

cd ../../
# Compile
cargo bundle --release

# Sign
#
codesign -s "University of Wisconsin-Madison" --strict --deep -f -v -o runtime --entitlements ./foray_ui/deploy_scripts/entitlements.plist ./target/release/bundle/osx/Foray.app/Contents/Macos/foray
#
codesign -s "University of Wisconsin-Madison" --strict --deep -f -v -o runtime --entitlements ./foray_ui/deploy_scripts/entitlements.plist ./target/release/bundle/osx/Foray.app
# Staple
xcrun stapler staple "./target/release/bundle/osx/Foray.app"
# Compress
cd ./target/release/bundle/osx
rm ./Foray*.zip
zip -r "$foray_file_zip"  ./Foray.app/

# credentials must already be stored via:
# xcrun notarytool store-credentials "notarytool-password" \
#                         --apple-id "johnnyc1423@gmail.com" \
#                         --team-id C658W77DY8 \
#                         --password <app specific password created from App-Specific Passwords" https://account.apple.com/account/manage>
#

# Notarize zip
xcrun notarytool submit "$foray_file_zip" --keychain-profile "notarytool-password" --wait

# Create a release and attach the zip
gh release create  "$1" --notes "$2" "$foray_file_zip"

# Update brew cask repo
numeric_version="${1:1}"
brew bump-cask-pr uw-mrtud/mrtud/foray --version "$numeric_version"

