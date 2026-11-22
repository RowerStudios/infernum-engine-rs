bin=$(sed -n '/^\[\[bin\]\]/,/^\[/s/^name *= *"\(.*\)"/\1/p' Cargo.toml | head -n 1) #detect binary name
./build/"$bin"
#removed mangohud cuz we have built in fps debug menu
#read -r -p "use mangohud? (y/n) " answer # prompt user for mangohud

#if [[ "$answer" =~ ^[Yy]$ ]]; then #if then statement
#    mangohud ./build/"$bin"
#else
#    ./build/"$bin"
#fi
