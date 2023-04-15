macro_rules! impl_op {
    ($op:ident, $method:ident, $fmt:literal, $prim:ident, f32, $out:ident) => {
        impl std::ops::$op<f32> for $prim {
            type Output = $out;

            fn $method(self, rhs: f32) -> Self::Output {
                $out(format!($fmt, self.0, rhs))
            }
        }
    };
    ($op:ident, $method:ident, $fmt:literal, $prim:ident, $other:ident, $out:ident) => {
        impl std::ops::$op<$other> for $prim {
            type Output = $out;

            fn $method(self, rhs: $other) -> Self::Output {
                $out(format!($fmt, self.0, rhs.0))
            }
        }
    };
}

macro_rules! impl_mul {
    ($prim:ident) => {
        impl_mul!($prim, $prim, $prim);
    };
    ($prim:ident, $other:ident) => {
        impl_mul!($prim, $other, $prim);
    };
    ($prim:ident, $other:ident, $out:ident) => {
        impl_op!(Mul, mul, "{} * {}", $prim, $other, $out);
    };
}

macro_rules! impl_add {
    ($prim:ident) => {
        impl_add!($prim, $prim, $prim);
    };
    ($prim:ident, $other:ident) => {
        impl_add!($prim, $other, $prim);
    };
    ($prim:ident, $other:ident, $out:ident) => {
        impl_op!(Add, add, "{} + {}", $prim, $other, $out);
    };
}

macro_rules! impl_sub {
    ($prim:ident) => {
        impl_sub!($prim, $prim, $prim);
    };
    ($prim:ident, $other:ident) => {
        impl_sub!($prim, $other, $prim);
    };
    ($prim:ident, $other:ident, $out:ident) => {
        impl_op!(Sub, sub, "{} - {}", $prim, $other, $out);
    };
}

macro_rules! impl_div {
    ($prim:ident) => {
        impl_div!($prim, $prim, $prim);
    };
    ($prim:ident, $other:ident) => {
        impl_div!($prim, $other, $prim);
    };
    ($prim:ident, $other:ident, $out:ident) => {
        impl_op!(Div, div, "{} / {}", $prim, $other, $out);
    };
}

macro_rules! impl_display {
    ($prim:ident) => {
        impl std::fmt::Display for $prim {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

pub mod prims {
    mod f32;
    pub use self::f32::*;
}
