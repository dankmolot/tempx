# TempX
A simple server for temporary storage of files.

Find an up-to-date Docker image here: [dankmolot/tempx](https://hub.docker.com/r/dankmolot/tempx)

## Table of Contents
* [What it does](#what-it-does)
* [Requirements](#requirements)
* [Development](#development)
* [Deployment](#deployment)
* [Configuration](#configuration)
* [API](#api)
* [Contributing](#contributing)
* [License](#license)

## What it does
Server for uploading and downloading temporary files with simple functionality.

## Requirements
* [NodeJS 14.x](https://nodejs.org)

## Development
To start the development server run:
```shell
npm install
npm start
```

Then, the service will be available here: http://localhost:8080

## Deployment
To deploy a docker container, run:
```
docker run -p 8080:8080 dankmolot/tempx:latest
```
P.S. You do not need to make a persistent volume, since each time you run `STORE_DIR` will be completely cleared

## Configuration
The server is configured using environment variables:

Variable | Default | Description
--- | --- | ---
PORT | `8080` | Server port
STORE_PATH | `./data` | Storage location for temporary files
LOG_LEVEL | `2` (info) | Logging level. See [loglevel](https://www.npmjs.com/package/loglevel#documentation)
UPLOAD_LIMIT | `5mb` | Maximum file upload limit. See [bytes](https://www.npmjs.com/package/bytes)

## API
The API documentation is [here](https://documenter.getpostman.com/view/16886665/TzseKSTh)

# Contributing
Pull requests are always welcome!

# License
[MIT](LICENSE)
