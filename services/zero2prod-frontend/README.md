This is the frontend component of Zero2Prod.

It uses yew

It is built using trunk:

```
trunk build
```

It can also be built from the root of the zero2prod project using xtask:

```
cargo xtask frontend
```

In that case, it creates a `dist` folder at the root the zero2prod, which
will be served statically by the zero2prod.


## Design

It starts with `Trunk.toml`

trunk sees `target = static/index.html`, so it opens that file to start. But
before building the site, it also sees in the `Trunk.toml` a hook 'pre_built',
which calls tailwindcss to build `static/css/main.css`.

In the index.html, it loads that css file, as well as the fonts directory:

```html
<head>
  <link data-trunk rel="css" href="./css/main.css" />
  <link data-trunk rel="copy-dir" href="./fonts" />
</head>
```

Finally it needs to build the application, and so it targets
the rust part in the crate directory:

```html
<body>
  <link data-trunk rel="rust" href="../crate/Cargo.toml"/>
</body>
```
