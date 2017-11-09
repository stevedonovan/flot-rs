extern crate flot;

fn main() {
    let line_data = vec![(0.0,1.0),(1.0,4.5)];
    let points_data = vec![(0.5,1.2),(0.8,4.0)];
 
    let page = flot::Page::new();
    
    let p = page.plot().size(500,300);
    p.lines("lines",line_data).fill(0.3).line_width(0);
    p.points("points",points_data).symbol("circle");
    
    page.render("simple.html").expect("i/o error");
}
