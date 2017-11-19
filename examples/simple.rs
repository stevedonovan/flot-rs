extern crate flot;

fn main() {
    let line_data = vec![(0.0,1.0),(1.0,4.5)];
    let points_data = vec![(0.5,1.2),(0.8,4.0)];

    let page = flot::Page::new("Lines and Points");

    let p = page.plot("").size(500,300);
    p.grid().color("red").background_gradient("#FFF","#AAA");
    p.xaxis().tick_values_and_labels(
        &[(0.0,"start"),(0.25,""),(0.5,"middle"),(0.75,""),(1.0,"end")]
    );
    p.lines("lines",line_data).fill(0.3).line_width(0);
    p.points("points",points_data).symbol("circle");
    p.text("
        Any descriptive text will be HTML escaped, so <bold>text<bold>
        doesn't work
    ");

    page.render("simple.html").expect("i/o error");
}
