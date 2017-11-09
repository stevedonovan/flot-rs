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
 
    let page = flot::Page::new();
    
    let p = page.plot();
    p.lines("lines",line_data).fill(0.3).line_width(0);
    p.points("points",points_data).symbol("circle");
    
    page.render("simple.html").expect("i/o error");
}

```

A `Page` may contain multiple plots; plots may contain multiple
series with chart types (`lines`,`points`,`bars`).

The result of running this program is to create 'simple.html', which
can be opened in your browser.

## Ways of specifying Data

By default, the series constructors take any iterator of `(f64,f64)` x-y
pairs. Note that the vectors `line_data` and `points_data` are consumed
by these calls.

Alternatively, can map a iterator of references with a function - `flot::mapr`
produces that points iterator, which here we collect into a vector.

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
    let page = flot::Page::new();

    let p = page.plot().size(500,300);
    let xvalues = flot::range(0.0,10.0,0.1).collect::<Vec<_>>();
    p.lines("norm s=1.0",make_gaussian(&xvalues,5.0,1.0));
    p.lines("norm s=0.7",make_gaussian(&xvalues,6.0,0.5));

    page.render("normal.html").unwrap();
}
```
`range` is a little convenience iterator for making ranges of floating-point
values (subsequently I've discovered that the `itertools_num` crate provides
something similar). 

`flot::mapv` is similar, except it takes an iterator of values. Here are the
squares of all integers from 0 to 9:

```rust
    page.plot().legend_pos(Corner::TopLeft)
        .bars("squares",mapv(0..10,|x| x*x))
        .width(0.75);  
```
(The iterator in `mapr` and `mapv` can provide any values which can be 
_converted_ into a `f64`, so this works.)

Finally, `flot::zip` can take two iterators of references, which are zipped
together into point tuples. This is useful if you have separate x and y data
as slices or vectors.

