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
 
    let page = Page::new();
    
    let p = page.plot();
    p.lines("lines",line_data).fill(0.3).line_width(0);
    p.points("points",points_data).symbol("circle");
    
    page.render("test.html").expect("i/o error");
}

```

A `Page` may contain multiple plots; plots may contain multiple
series with chart types (`lines`,`points`,`bars`).

The result of running this program is to create 'test.html', which
can be opened in your browser.

