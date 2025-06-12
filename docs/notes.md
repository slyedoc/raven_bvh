# Notes

## Bench

```bash
 cargo bench --features camera 
```

To save to compare:

```bash
cargo bench --features camera > ./tmp/base
```

Make change, then:

```bash
cargo bench --features camera > ./tmp/change
```

Then to compare with [benchcmp](https://github.com/BurntSushi/cargo-benchcmp):

```bash
cargo benchcmp ./tmp/base ./tmp/change      
```

