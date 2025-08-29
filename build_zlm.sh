git clone --depth 1 https://github.com/ZLMediaKit/ZLMediaKit.git
cd ZLMediaKit
git submodule update --init --recursive
mkdir build
cd build
rm -rf ./installer
cmake .. -DENABLE_WEBRTC=true -DCMAKE_INSTALL_PREFIX=./installer
make -j4 && make install
