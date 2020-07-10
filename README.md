# Proof of Concept for rust-bio-types GFF Serializer Rewrite

This is a proof of concept for the `bio-types` rewrite. This would replace
the internal parser with `csv` and eliminate the custom getter currently used
in `bio::io::gff`.

## Author

* [Yishen Miao](https://github.com/mys721tx)
