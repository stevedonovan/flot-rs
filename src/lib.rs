// flot test
extern crate typed_arena;
use typed_arena::Arena;

#[macro_use]
extern crate json;
use json::JsonValue;

use std::io;
use std::io::Write;

/// Iterator type for floating-point range iterator
pub struct FRange {
    val: f64,
    end: f64,
    incr: f64
}

/// generates an iterator between `x1` and `x2`, step `skip`
/// over floating point numbers.
pub fn range(x1: f64, x2: f64, skip: f64) -> FRange {
    FRange {val: x1, end: x2, incr: skip}
}

impl Iterator for FRange {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.val;
        if res >= self.end {
            None
        } else {
            self.val += self.incr;
            Some(res)
        }
    }
}

/// join two iterators of references together to produce point tuples.
/// The reference types can be anything that converts to `f64`
pub fn zip<'a,I1,I2,T1,T2>(x: I1, y: I2) -> Box<Iterator<Item=(f64,f64)>+'a> 
where I1: IntoIterator<Item=&'a T1>+'a, I2: IntoIterator<Item=&'a T2>+'a,
    T1: Into<f64>+Copy+'a, T2: Into<f64>+Copy+'a
{
    Box::new(x.into_iter().zip(y).map(|(&x,&y)| (x.into(),y.into())))
}

/// map an iterator of references with a function producing point tuples.
/// Like `zip`, the reference type can be anything that converts to `f64`
pub fn mapr<'a,I,T,F>(x: I, f: F) -> Box<Iterator<Item=(f64,f64)>+'a> 
where I: IntoIterator<Item=&'a T>+'a, F: Fn(f64)->f64 + 'a,
    T: Into<f64>+Copy+'a
{
    Box::new(x.into_iter().map(move |&x| { let fv = x.into(); (fv,f(fv))}))
}

/// map an iterator of values with a function producing point tuples.
/// The value type can be anything that converts to `f64`
pub fn mapv<'a,I,T,F>(x: I, f: F) -> Box<Iterator<Item=(f64,f64)>+'a> 
where I: IntoIterator<Item=T>+'a, F: Fn(f64)->f64 + 'a,
    T: Into<f64>+Copy+'a
{
    Box::new(x.into_iter().map(move |x| { let fv = x.into(); (fv,f(fv))}))
}


enum PlotKind {
    Lines,
    Points,
    Bars
}

impl PlotKind {
    fn to_str(&self) -> &'static str {
        match *self {
            PlotKind::Lines => "lines",
            PlotKind::Points => "points",
            PlotKind::Bars => "bars"
        }
    }
}

/// describes a data series which can be plotted either as lines, points or bars
pub struct Series {
    data: JsonValue,
    kind: PlotKind,
    symbols: bool,
}

impl Series {
    fn new<T>(kind: PlotKind, label: &str, data: T) -> Series
    where T: IntoIterator<Item=(f64,f64)> {

        let mut arr = JsonValue::new_array();
        for p in data.into_iter() {
            arr.push(array![p.0,p.1]).unwrap();
        }
        let mut data = object! {
            "label" => label,
            "data" => arr
        };
        data[kind.to_str()] = object!{"show" => true};
        Series {data: data, kind: kind, symbols: false}
    }

    fn kind_ref(&mut self) -> &mut JsonValue {
        &mut self.data[self.kind.to_str()]
    }
    
    /// set the xaxis for this series (2 for second)    
    pub fn xaxis(&mut self, which: u32) -> &mut Self {
        self.data["xaxis"] = which.into();
        self        
    }
    
    /// set the yaxis for this series (2 for second)    
    pub fn yaxis(&mut self, which: u32) -> &mut Self {
        self.data["yaxis"] = which.into();
        self        
    }

    /// set the fill colour underneath lines or in bars as an alpha value.
    pub fn fill(&mut self, opacity: f32) -> &mut Self {
        self.kind_ref()["fill"] = opacity.into();
        self
    }
    
    /// set the fill colour underneath lines or in bars as an HTML colour.
    pub fn fill_color(&mut self, color: &str) -> &mut Self {
        self.kind_ref()["fillColor"] = color.into();
        self
    }
    
    /// set the line colour as an HTML colour.
    pub fn color(&mut self, color: &str) -> &mut Self {
        self.data["color"] = color.into();
        self
    }    

    /// width of the line - zero for no shadow.
    pub fn line_width(&mut self, size: u32) -> &mut Self {
        self.kind_ref()["lineWidth"] = size.into();
        self
    }

    /// radius for points (points only)
    pub fn radius(&mut self, size: u32) -> &mut Self {
        match self.kind {
            PlotKind::Points => self.kind_ref()["radius"] = size.into(),
            _ => panic!("radius() only applies to points")
        }
        self
    }

    /// symbol for points (points only)
    pub fn symbol(&mut self, name: &str) -> &mut Self {
        match self.kind {
            PlotKind::Points => {
                self.symbols = true;
                self.kind_ref()["symbol"] = name.into();
            },
            _ => panic!("symbol() only applies to points")
        }
        self
    }
    
    /// draw steps between points (lines only)    
    pub fn steps(&mut self) -> &mut Self {
        match self.kind {
            PlotKind::Lines => self.kind_ref()["steps"] = true.into(),
            _ => panic!("steps() only applies to lines")
        }
        self
    }

    /// set width of bars (bars only)
    pub fn width(&mut self, width: f64) -> &mut Self {
        match self.kind {
            PlotKind::Bars => self.kind_ref()["barWidth"] = width.into(),
            _ => panic!("bar_width() only applies to bars")
        }
        self
    }



}

/// describes position of legend (None for no legend)
pub enum Corner {
    None,
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

impl Corner {
    fn to_str(&self) -> &'static str {
        use Corner::*;
        match *self {
            None => "none",
            TopRight => "ne",
            TopLeft => "nw",
            BottomRight => "sw",
            BottomLeft => "se",
        }
    }
}

/// describes sides of plot for axis position
pub enum Side {
    Right,
    Left,
    Bottom,
    Top,
}

impl Side {
    fn to_str(&self) -> &'static str {
        use Side::*;
        match *self {
            Right => "right",
            Left => "left",
            Bottom => "bottom",
            Top => "top",
        }
    }
}

/// represents an axis
pub struct Axis<'a> {
    which: &'static str,
    plot: &'a mut Plot,
    idx: usize,
}

impl <'a> Axis<'a> {
    fn new(which: &'static str, plot: &'a mut Plot, idx: usize) -> Axis<'a> {
        if plot.options[which].is_null() {
            plot.options[which] = array![object!{}];
        }
        if idx > 1 {
            plot.options[which].push(object!{}).unwrap();
        }
        Axis{which: which, plot: plot, idx: idx-1}
    }

    pub fn set_option(&mut self, key: &str, val: JsonValue) -> &mut Self {
        self.plot.options[self.which][self.idx][key] = val;
        self
    }
    
    /// force minimum value on axis
    pub fn min(&mut self, min: f64) -> &mut Self {
        self.set_option("min",min.into());
        self
    }
    
    /// force maximum value on axis
    pub fn max(&mut self, max: f64) -> &mut Self {
        self.set_option("max",max.into());
        self
    }  

    /// set both minimum and maximum on axis
    pub fn bounds(&mut self, min: Option<f64>, max: Option<f64>) -> &mut Self {
        if let Some(min) = min {
            self.min(min);
        }
        if let Some(max) = max {
            self.max(max);
        }
        self
    }
    
    /// set the position of an axis
    pub fn position(&mut self, side: Side) -> &mut Self {
        let pos = side.to_str();
        if pos == "right" {
            self.set_option("alignTicksWithAxis",1.into());
        }
        self.set_option("position",pos.into())
    }

    /// indicates that this axis uses time values.
    /// These must be in milliseconds
    pub fn time(&mut self) -> &mut Self {
        self.plot.time = true;
        self.set_option("mode","time".into());
        self
    }

}

/// represents 'markings' or plot annotations.
pub struct Markings<'a> {
    plot: &'a mut Plot,
}

impl <'a> Markings<'a> {
    fn new(plot: &'a mut Plot) -> Markings<'a> {
        plot.set_option("grid","markings",array![]);
        Markings{plot: plot}
    }
    
    fn markings(&mut self) -> &mut JsonValue {
        &mut self.plot.options["grid"]["markings"]
    }
    
    pub fn add_marking(&mut self, val: JsonValue) -> &mut Self {
        self.markings().push(val).unwrap(); // it must be an array
        self
    }
    
    /// vertical band over plot
    pub fn vertical_area(&mut self, p1: f64, p2: f64) -> &mut Self {
        self.add_marking(object!{"xaxis" => object!{"from"=>p1,"to"=>p2 } })
    }
    
    /// horizontal band over plot
    pub fn horizontal_area(&mut self, p1: f64, p2: f64) -> &mut Self {
        self.add_marking(object!{"yaxis" => object!{"from"=>p1,"to"=>p2 } })
    }
    
    /// vertical line at x position
    pub fn vertical_line(&mut self, pos: f64) -> &mut Self {
        self.vertical_area(pos,pos)
    }
    
    /// horizontal line at y position
    pub fn horizontal_line(&mut self, pos: f64) -> &mut Self {
        self.horizontal_area(pos,pos)
    }
    
    /// rectangular area specified as (x1,x2,y1,y2)
    pub fn area(&mut self, x1: f64, x2: f64, y1: f64, y2: f64) -> &mut Self {
        self.add_marking(object!{
            "xaxis" => object!{"from"=>x1,"to"=>x2 },
            "yaxis" => object!{"from"=>y1,"to"=>y2 }
        })
    }

    /// set the color of the last marking defined
    pub fn color(&mut self, color: &str) -> &mut Self {
        {    
            let mut arr = self.markings();
            let len = arr.len();
            arr[len-1]["color"] = color.into();
        }
        self
    }

}

/// represents a particular plot
pub struct Plot {
    series: Arena<Series>,
    placeholder: String,
    options: JsonValue,
    time: bool,
    symbols: bool,
    bounds: (u32,u32),
}

impl Plot {
    fn new(name: &str) -> Plot {
        Plot {
            series: Arena::new(),
            placeholder: name.into(),
            options: object!{},
            time: false,
            symbols: false,
            bounds: (800,300),
        }
    }
    
    /// the size in pixels (width,height) of the plot area
    pub fn size(&mut self,width:u32,height:u32) -> &mut Self {
        self.bounds = (width,height);
        self
    }

    /// x axis object
    pub fn xaxis<'a>(&'a mut self) -> Axis<'a> {
        Axis::new("xaxes",self,1)
    }

    /// y axis object
    pub fn yaxis<'a>(&'a mut self) -> Axis<'a> {
        Axis::new("yaxes",self,1)
    }
    
    /// second y axis object
    pub fn yaxis2<'a>(&'a mut self) -> Axis<'a> {
        Axis::new("yaxes",self,2)
    }    

    /// create a data series with individual points
    /// The data is anything that converts to an iterator
    /// of `(f64,f64)` tuples
    pub fn points<T>(&self, label: &str, data: T) -> &mut Series
    where T: IntoIterator<Item=(f64,f64)> {
        self.series.alloc(Series::new(PlotKind::Points,label,data))
    }

    /// create a data series joined with lines
    pub fn lines<T>(&self, label: &str, data: T) -> &mut Series
    where T: IntoIterator<Item=(f64,f64)> {
        self.series.alloc(Series::new(PlotKind::Lines,label,data))
    }

    /// create a data series with bars (histogram).
    pub fn bars<T>(&self, label: &str, data: T) -> &mut Series
    where T: IntoIterator<Item=(f64,f64)> {
        self.series.alloc(Series::new(PlotKind::Bars,label,data))
    }

    /// position of axis (Corner::None to hide)
    pub fn legend_pos(&mut self, pos: Corner) -> &mut Self {
        let obj = match pos {
            Corner::None => object!{ "show" => false },
            _ => object!{ "position" => pos.to_str() }
        };
        self.options["legend"] = obj;
        self
    }

    pub fn extra_symbols(&mut self) -> &mut Self {
        self.symbols = true;
        self
    }
    
    /// object to create markings like lines and areas
    pub fn markings<'a>(&'a mut self) -> Markings<'a> {
        Markings::new(self)
    }

    pub fn set_option(&mut self, key: &str, subkey: &str, val: JsonValue) -> &mut Self {
        if self.options[key].is_null() {
            self.options[key] = object!{};
        }
        self.options[key][subkey] = val;
        self
    }

    fn render_placeholder(&self) -> String {
        format!("<div id={:?} style=\"width:{}px;height:{}px\"></div>\n",
            self.placeholder,self.bounds.0,self.bounds.1)
    }

    fn render_script(self, f: &mut Write) -> io::Result<()> {
        let series = self.series.into_vec();
        let mut data = '['.to_string();
        let basename = "plot";
        let mut k = 1;
        for s in &series {
            let varname = format!("{}_{}",basename,k);
            k += 1;
            write!(f,"var {} = {};\n",varname,s.data)?;
            data += &varname;
            data.push(',');
        }
        data.pop();
        data.push(']');
        write!(f,"$.plot($(\"#{}\"),{},{});\n",self.placeholder,data,self.options)
    }

}

use std::env;
use std::fs::File;
use std::mem;
use std::cell::Cell;

/// represents a collection of plots
pub struct Page {
    plots: Arena<Plot>,
    count: Cell<usize>,
}

fn script(base: &str, name: &str) -> String {
    format!("<script language=\"javascript\" type=\"text/javascript\" src=\"{}/{}\"></script>",
        base,name)
}

impl Page {
    pub fn new() -> Page {
        Page {
            plots: Arena::new(),
            count: Cell::new(0)
        }
    }

    /// create a new plot
    pub fn plot(&self) -> &mut Plot {
        let count = &self.count;
        count.set(count.get() + 1);
        let name = format!("plot{}",self.count.get());
        self.plots.alloc(Plot::new(&name))
    }

    /// render the page as HTML to the given file
    pub fn render(&self, file: &str) -> io::Result<()> {
        // this is deeply dubious. In an ideal world with non-lexical lifetimes,
        // this could be a self method, since it is _only_ called after all
        // the plots have been defined. It cannot be &mut self, because borrows
        // are already happening, so we do a &Self -> &mut Self conversion.
        //
        // In an even more ideal world, typed_arena::Arena would have an
        // iterator over references
        let mut nplots = Arena::new();
        let self_ptr: *const Self = self;
        let mut_self: &mut Self = unsafe { &mut * (self_ptr as *mut Self) };
        mem::swap(&mut mut_self.plots, &mut nplots);

        let plots = nplots.into_vec();

        let (jquery,flot) = if let Ok(f) = env::var("FLOT") {
            let local = format!("file://{}",f);
            (local.clone(),local.clone())
        } else {
            (
                "https://cdnjs.cloudflare.com/ajax/libs/jquery/3.2.1".to_string(),
                "https://cdnjs.cloudflare.com/ajax/libs/flot/0.8.3".to_string()
            )
        };
        let mut f = File::create(file)?;
        let header = "
<html>
 <head>
    <meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\">
    <title>Flot Examples</title>
";
        write!(f,"{}{}\n{}\n",header,
            script(&jquery,"jquery.min.js"),
            script(&flot,"jquery.flot.min.js"))?;
        if plots.iter().any(|p| p.time) {
            write!(f,"{}\n",script(&flot,"jquery.flot.time.min.js"))?;
        }
        if plots.iter().any(|p| p.symbols) {
            write!(f,"{}\n",script(&flot,"jquery.flot.symbol.min.js"))?;
        }
        write!(f,"</head>\n</body>\n")?;
        for p in &plots {
            write!(f,"{}\n",p.render_placeholder())?;
        }
        write!(f,"<script type=\"text/javascript\">\n$(function () {{\n")?;
        for p in plots {
            p.render_script(&mut f)?;
        }
        write!(f,"}});\n</script>\n</body>\n</html>\n")
    }
}

