# A Rust library for generating Flot documents

[Flot](http://www.flotcharts.org/) is a JavaScript library for generating
attractive data plots.  Although usually used to enhance interactive
websites, **flot-rs** is a nice way for command-line programs to create
standalone HTML documents with plots. By default these refer to online
sources, so they can be handed over to anybody else for display.

```rust
extern crate flot;

fn main() {
    let line_data = vec![(0.0,1.0),(1.0,4.5)];
    let points_data = vec![(0.5,1.2),(0.8,4.0)];

    let page = flot::Page::new("");

    let p = page.plot("Lines and Points");
    p.lines("lines",line_data).fill(0.3).line_width(0);
    p.points("points",points_data).symbol("circle");

    page.render("simple.html").expect("i/o error");
}

```

A `Page` may contain multiple plots; plots may contain multiple
series with chart types (`lines`,`points`,`bars`).

The result of running this program is to create 'simple.html', which
can be opened in your browser.

`Page` can be given a title, which if non-empty will both set the title
of the document and create a H1 heading. Likewise, the `plot` method is
given a title which if non-empty will provide a centered H2 heading for
the plot.

## Ways of specifying Data

By default, the series constructors take anything that converts to an
iterator of `(f64,f64)` x-y pairs.
Note that the vectors `line_data` and `points_data` are consumed
by these calls.

If you have a source of tuples that isn't `(f64,f64)`, then
`flot::to_f64` will convert that into a form that _flot-rs_ accepts, provided
that those types convert cleanly into `f64`.

Alternatively, you can map a iterator of references with a function - `flot::mapr`
produces the desired points iterator, which here we collect into a vector.

```rust
extern crate flot;

fn make_gaussian(xvalues: &[f64], m: f64, s: f64) -> Vec<(f64,f64)> {
    use std::f64::consts::PI;
    let s2 = 2.0*s*s;
    let norm = 1.0/(s2*PI).sqrt();
    flot::mapr (
        xvalues,
        move |x| norm*(-(x-m).powi(2)/s2).exp()
    ).collect()
}

fn main() {
    let page = flot::Page::new("");

    let p = page.plot("Normal distribution").size(500,300);
    let xvalues: Vec<_> = flot::range(0.0,10.0,0.1).collect();
    p.lines("norm σ=1.0",make_gaussian(&xvalues,5.0,1.0));
    p.lines("norm σ=0.7",make_gaussian(&xvalues,6.0,0.5));

    page.render("normal.html").unwrap();
}
```
`range` is a little convenience iterator for making ranges of floating-point
values (subsequently I've discovered that the [itertools-num](https://docs.rs/itertools-num)
crate provides something similar - see `linspace`).


`flot::mapv` is similar, except it takes an iterator of _values_. Here are the
squares of all integers from 0 to 9:

```rust
    page.plot().legend_pos(Corner::TopLeft)
        .bars("squares",mapv(0..10,|x| x*x))
        .width(0.75);
```
(The iterator given to `mapr` and `mapv` can provide any values which can be
_converted_ into a `f64`, so the integer range works.)

Finally, `flot::zip` can take two iterators of references, which are zipped
together into point tuples. This is useful if you have separate x and y data
as slices or vectors.

## Using _flot-rs_ as a Personal Display Engine

By default, _flot-rs_ uses the Cloudflare CDN for jQuery (3.2.1) and Flot (0.8.3),
which means that these HTML documents are portable and can be viewed with anyone
with an internet connenction. Browsers cache these dependencies, so that generally
these documents render quickly. However, if you download Flot directly, then
you can set the environment variable `FLOT` to its location. E.g. I have
`export FLOT=/home/steve/Downloads/flot`.

Being a command-line person, I tend to open generated HTML documents using
the appropriate command, `start` for Windows, `open` for MacOS, `gnome-open`
for Linux. There are browser-specific options for opening documents without
toolbars and such like in their own window, e.g. `google-chrome --app=doc.html`
and `firefox --chrome doc.html` for Firefox.

