extern crate flot;
use flot::*;
fn main() {
    let page = Page::new();
    
    page.plot().legend_pos(Corner::TopLeft)
        .bars("squares",mapv(0..10,|n| n*n as f64))
        .width(0.75);  
    
    page.render("squares.html").expect("i/o error");


}
