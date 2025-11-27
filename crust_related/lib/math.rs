// math_utils.rs
#![no_std]

pub mod math {
    pub trait Numeric: Copy + PartialOrd {
        fn zero() -> Self;
        fn one() -> Self;
        fn from_u32(n: u32) -> Self;
    }
    
    impl Numeric for i32 {
        fn zero() -> Self { 0 }
        fn one() -> Self { 1 }
        fn from_u32(n: u32) -> Self { n as i32 }
    }
    
    impl Numeric for f32 {
        fn zero() -> Self { 0.0 }
        fn one() -> Self { 1.0 }
        fn from_u32(n: u32) -> Self { n as f32 }
    }
    
    /// Greatest common divisor
    pub fn gcd<T>(mut a: T, mut b: T) -> T 
    where
        T: Numeric + core::ops::Rem<Output = T>,
    {
        while b != T::zero() {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a
    }
    
    /// Least common multiple
    pub fn lcm<T>(a: T, b: T) -> T 
    where
        T: Numeric + core::ops::Mul<Output = T> + core::ops::Div<Output = T>,
    {
        (a * b) / gcd(a, b)
    }
    
    /// Integer square root (Babylonian method)
    pub fn isqrt(n: u32) -> u32 {
        if n == 0 { return 0; }
        
        let mut x = n;
        let mut y = (x + 1) / 2;
        
        while y < x {
            x = y;
            y = (x + n / x) / 2;
        }
        
        x
    }
    
    /// Power function for integers
    pub fn pow<T>(base: T, exponent: u32) -> T 
    where
        T: Numeric + core::ops::Mul<Output = T>,
    {
        if exponent == 0 {
            return T::one();
        }
        
        let mut result = base;
        for _ in 1..exponent {
            result = result * base;
        }
        result
    }
    
    /// Factorial
    pub fn factorial(n: u32) -> Option<u64> {
        if n == 0 { return Some(1); }
        
        let mut result: u64 = 1;
        for i in 1..=n {
            result = result.checked_mul(i as u64)?;
        }
        Some(result)
    }
    
    /// Check if number is prime
    pub fn is_prime(n: u32) -> bool {
        if n <= 1 { return false; }
        if n <= 3 { return true; }
        if n % 2 == 0 || n % 3 == 0 { return false; }
        
        let mut i = 5;
        while i * i <= n {
            if n % i == 0 || n % (i + 2) == 0 {
                return false;
            }
            i += 6;
        }
        true
    }
}
