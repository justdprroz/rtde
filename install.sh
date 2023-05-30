echo "1. Installing window manager"
cd rtde-wm
echo "2. Building project in debug mode"
cargo build
echo "3. Copying binary to /usr/local/bin"
sudo cp -f ./target/debug/rtwm /usr/local/bin

echo
echo "Installation completed!"
echo "Now you can add \"exec rtwm\" to your .xinitrc"
