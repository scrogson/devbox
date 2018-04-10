# devbox

> devbox is responsible for bootstrapping your development environment.

## About devbox

`devbox` is a work in progress. It is an opinionated wrapper around `docker` and
`docker-compose`.

`devbox` provides a way to get all of your services for local application
development up and running quickly. Simply create a project and tell `devbox`
what services are included, it will handle the rest.

## Installation

In order to build this package you'll need a few dependencies install:

* Rust
* Docker

### Installing Rust

The de-facto way to install [Rust](https://www.rust-lang.org/en-US/) is via [rustup](https://rustup.rs/).

Run the following in your terminal, then follow the onscreen instructions.

```shell
$ curl https://sh.rustup.rs -sSf | sh
```

### Installing Docker

If you're running macOS you'll want visit the [Docker for Mac website](https://store.docker.com/editions/community/docker-ce-desktop-mac) for
installation instructions.

### Installing devbox

To get `devbox` installed, simply run the following:

```shell
$ cargo install --git https://github.com/scrogson/devbox --bin devbox
```

This will compile `devbox` and move the resulting executable into `~/.cargo/bin`
which you should have been instructed to put in your `$PATH` when installing
Rust.

### Generating a Project

`devbox` works with projects. A project is a namespace to provide isolation
between containers in other projects one might have.

To generate a project, use the `new` subcommand with the name of our project like so:

```shell
$ devbox new example
```

This will generate a couple of files in your home directory:

```shell
$ tree ~/.config/devbox/ -L 2
~/.config/devbox/
└── example
    ├── config.toml
    └── docker-compose.yml

1 directory, 2 files
```

The `config.toml` file is the `devbox` configuration for your project. In it
contains configuration for all of the services in your project. It looks like
this:

```toml
volumes = [
  "mysql",
  "postgres"
]

[services]
example = { git = "git@github.com:user/example" }
```

#### Volumes

Volumes is an array of `docker` volume names used in your project. These volumes
will be created by `devbox build`.

#### Services

Services are declared by specifying the name of the service and configuring
a `git` repository URL where the service can be cloned from. When working with
a local service on disk, specify the `path` option along with the absolute path
to the service on disk.

### Build the Docker Containers

Set up the networking, pull down the latest docker images, and build the docker
containers:

```shell
$ devbox build -p example
```

## Running devbox

From the root of the repository:

```shell
$ devbox start -p example
```

This will run the support services.

The `ps` command can be used to list the running containers and confirm they have started correctly:

```shell
λ devbox ps
CONTAINER ID        NAMES                    STATUS              PORTS
8fd9e21e74dc        example_kafka_1          Up 6 minutes        127.0.0.1:9092->9092/tcp
66520b2ba9cc        example_zookeeper_1      Up 6 minutes        2888/tcp, 127.0.0.1:2181->2181/tcp, 3888/tcp
f8f37e7baa69        example_mysql_1          Up 6 minutes        127.0.0.1:3306->3306/tcp
ec04a7699e15        example_postgres_1       Up 6 minutes        127.0.0.1:5432->5432/tcp
45abb5cdd9ad        example_elasticsearch_1  Up 6 minutes        127.0.0.1:9200->9200/tcp, 9300/tcp
fe8b9d71bf70        example_redis_1          Up 6 minutes        127.0.0.1:6379->6379/tcp
```

### Reading Logs

Each service runs in its own docker container and writes its logs to standard
output. In order to view the latest log output:

```shell
$ docker logs -p example mysql
```

To stream the logs in real time use the `--follow` flag:

```shell
$ docker logs -p example -f postgres
```

**Note:** use `docker ps` to see a list of docker container names.

### Stopping devbox

From the root of the repository:

```shell
$ devbox stop -p example
```

This will stop the docker containers in the `example` project.

## Troubleshooting

### Failure Starting

Ensure the Docker for Mac application is running.

To always start on boot OS X users can go to `System Preferences > Users
& Groups` and add the Docker application to the Login Items list.

### Services in Status Restarting (137)

This is likely caused by Docker not having access to enough memory. You can
change this in Docker preferences in the `Advanced` tab. By default Docker is
set to request 2GB of memory. You may need to bump this to at least 4GB in order
to run all services provided by `devbox`.

### Problems connecting to MySQL

If using Docker for Mac, the MySQL instance is reachable on `127.0.0.1` on the
default `3306` port.

    mysql -h 127.0.0.1 -u root -p

The PostgreSQL container creates a `postgres` user with a password of `postgres`.
See the `.env` for the latest values.

### Problems connecting to PostgreSQL

If using Docker for Mac, the PostgreSQL instance is reachable on `127.0.0.1` on the
default `5432` port.

    psql -h 127.0.0.1 -U postgres

The PostgreSQL container creates a `postgres` user with a `postgres` password.
See the `.env` for the latest values.
