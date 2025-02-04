# rsLib
A collection of some rust functions

Subjects that this library covers:
- strings

Strings:
- utilities to manipulate strings

# Run Tests
## Native
```bash
cd ./rs_lib
cargo test
```

## Docker or Podman
### Docker
```bash
docker build -t rslib_image .
docker run --replace --name rslib_container rslib_image
```

### Podman
```bash
podman build -t rslib_image .
podman run --replace --name rslib_container rslib_image
```
