cur_dir=$PWD
cd ..

app_name=noted

mkdir -p release/win
mkdir -p release/linux

target=x86_64-pc-windows-gnu
cargo build --target $target --release

cp debug/$target/release/$app_name.exe release/win/$app_name.exe

target=x86_64-unknown-linux-gnu
cargo build --target $target --release

cp debug/$target/release/$app_name release/linux/$app_name

cd $cur_dir