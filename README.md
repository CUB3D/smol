# SMOL
### Simple Minimizer Of Links
A simple link shortener, written in Rust with actix-web
![Licence](https://img.shields.io/github/license/CUB3D/smol)

### Screenshots
![Screenshot](./doc/screen.png)

### Try it out
Current git master hosted -> [here](https://s.cub3d.pw)

### Running with docker
```yaml
version: '3'
services:
 smol:
   container_name: smol
   build:
     context: https://github.com/CUB3D/smol.git
   ports:
     - "8094:8080"
   environment:
     RUST_LOG: info
     DATABASE_URL: "<TODO>"
   restart: unless-stopped
```
