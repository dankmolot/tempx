# TempX
Small and simple server for temporary serving files in one binary.

# Motivation
In the game [Garry's mod](https://store.steampowered.com/app/4000/), i needed to transfer files between clients. 
Garry's mod have it is own network stream, but speed is very limited, and file with size ~3mb could transfer about 30 seconds. 
So i wrote this small application to store this files, and get them.

# Planned things
* Secure uploaded file with random access key
* Rate limit
* Automated builds

# Documentation
When you deploy TempX, you can access index page http://localhost:3000/, to see documentation.

You can also see documentation at [source code](src/main.rs#L26)

# Configuration
You can configure server with enviroment variables
```
APP_PORT=3000
APP_DEFAULT_EXPIRE=1m
APP_MAX_EXPIRE=10m
APP_LIMITS.FILE=10MiB
APP_ADDRESS=0.0.0.0
```

or with `App.toml` configuration file
```toml
port=3000
default_expire="1m"
max_expire="10m"
limits.file="10MiB"
address="0.0.0.0"
```

# License
[MIT](LICENSE)