# Docker Setup

Run uwu-admin with Docker or Docker Compose.

## Docker Compose

From the repository root:

```sh
cp docker-compose.example.yml docker-compose.yml
docker compose up -d
```

## Manual Docker Build

```sh
docker build -f containers/docker/Dockerfile -t uwu-admin .
docker run -d -p 8080:80 -v uwu-data:/data --name uwu-admin uwu-admin
```

## Manage

```sh
docker compose logs -f    # view logs
docker compose restart    # restart
docker compose down       # stop and remove containers
```

The admin panel will be available at `http://localhost:8080`.
