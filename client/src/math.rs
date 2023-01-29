pub trait Point2Extend {
    fn length(&self) -> f64;
}

impl Point2Extend for cgmath::Point2<f64> {
    fn length(&self) -> f64 {
        return (self.x * self.x + self.y * self.y).sqrt();
    }
}

impl Point2Extend for cgmath::Vector2<f64> {
    fn length(&self) -> f64 {
        return (self.x * self.x + self.y * self.y).sqrt();
    }
}