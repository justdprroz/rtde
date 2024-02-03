# Installing base packages
echo ">>> Installing packages"
sudo pacman --needed --noconfirm -S \
  xorg xorg-xinit nvidia kitty picom polybar dmenu ttf-cascadia-code scrot \

# Ensure cargo is installed
echo ">>> Checking rust installation"
if ! command -v cargo &> /dev/null
then
  echo ">>> Installing"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source "$HOME/.cargo/env"
else
  echo ">>> Rust is present"
fi 

# Install and configure rtde
echo ">>> Installing rtwm"
# cd rtde-wm
# cargo build
# sudo cp -f ./target/debug/rtwm /usr/local/bin
cargo install --debug --path rtde-wm


# Copy help script 
echo ">>> Copying scripts"
sudo cp ./scripts/* ~/.cargo/bin/ 

# create rtde dir and add autostart file
echo ">>> Creating config directory"
mkdir ~/.rtde
touch ~/.rtde/out.log
touch ~/.rtde/err.log

# create autostart config
cp autostart.sh ~/.rtde/autostart.sh
chmod +x ~/.rtde/autostart.sh

mkdir ~/Pictures/screenshots -p

# Backup xinitrc and create new
echo ">>> Updating xinitrc"
mv ~/.xinitrc ~/.xinitrc.old
echo -e "exec startrtde" > ~/.xinitrc


