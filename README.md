# pyDistrib-CLI

PyDistrib is a python library to speed up computations by leveraging distributed computing. This CLI tool initiates a worker node.

## Usage

1. Start one or more worker nodes:

```bash
# TODO
```

2. Start a [PyDistrib](https://github.com/rayanht/pyDistrib) server from Python:

```python
with PyDistribServer(port=5000).start() as s:
  s.submit(fn=test_fn, args=[x,y,z,])
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.
