
# Multiplayer Shooter

A simple 2d shooter using [Actix Web](https://actix.rs) on the backend and [Rapier2d](https://rapier.rs) as the physics engine

![Alt Text](https://github.com/Andrew-Y-Xia/multiplayer_shooter/blob/master/static/game_demo.gif)



## Run


An installation of the rust toolchain is required. Assuming Unix-like OS:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Clone repo:
```bash
git clone https://github.com/Andrew-Y-Xia/multiplayer_shooter.git
cd ./multiplayer_shooter
```
Build project; this step will take ~3min.
```bash
cargo build --release
```
Run:
```bash
cargo run --release
```
Go to http://0.0.0.0:8080. If you want to access the site from another
computer on the local network, go to http://{HOSTNAME}:8080 instead. 
You can find HOSTNAME on MacOS by:
```bash
hostname
```
