extern crate flot;
use flot::*;
fn main() {
    let page = Page::new("Histogram");

    page.plot("Squares of Integers up to 9").legend_pos(Corner::TopLeft)
        .bars("squares",mapv(0..10,|x| x*x))
        .width(0.75);

    page.render("squares.html").expect("i/o error");


}
