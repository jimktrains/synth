use std::fmt;
use std::ops::Mul;
use std::ops::MulAssign;

#[derive(Eq, PartialEq, Clone, Ord, PartialOrd)]
pub struct SQ1_7(pub i8);
#[derive(Eq, PartialEq, Clone, Ord, PartialOrd)]
pub struct SQ8_0(pub i8);
#[derive(Eq, PartialEq, Clone, Ord, PartialOrd)]
pub struct SQ16_0(pub i16);
#[derive(Eq, PartialEq, Clone, Ord, PartialOrd)]
pub struct SQ1_15(pub i16);
#[derive(Eq, PartialEq, Clone, Ord, PartialOrd)]
pub struct SQ8_8(pub i16);

#[derive(Eq, PartialEq, Clone, Ord, PartialOrd)]
pub struct SQ1_31(pub i32);
#[derive(Eq, PartialEq, Clone, Ord, PartialOrd)]
pub struct SQ16_15(pub i32);

#[derive(Eq, PartialEq, Clone, Ord, PartialOrd)]
pub struct SQ32_0(pub i32);

impl SQ32_0 {
    pub const fn inv_u16(n: u16) -> SQ1_31 {
        SQ32_0::div(SQ32_0(1), SQ32_0(n as i32))
    }

    pub const fn div(on: SQ32_0, od: SQ32_0) -> SQ1_31 {
        let mut neg = 1;
        let mut on = on.0;
        let mut od = od.0;
        if on < 0 {
            on *= -1;
            neg *= -1;
        }
        if od < 0 {
            od *= -1;
            neg *= -1;
        };
        let on = on;
        let od = od;
        let neg = neg;

        let mut r: u32 = 0;
        let mut c: usize = 0;

        // let mut n: u32 = on;
        // let mut d: u32 = od;
        // let mut n_mag: usize = 1;
        // let mut d_mag: usize = 1;

        // for _ in 0..32 {
        //     d >>= 1;
        //     if d != 0 {
        //         d_mag += 1;
        //     }

        //     n >>= 1;
        //     if n != 0 {
        //         n_mag += 1;
        //     }
        // }

        let mut n = on.reverse_bits();
        let mut t = 0;
        let d = od;

        while t <= d {
            let lsb = n & 0x1;
            t = (t << 1) + lsb;
            n >>= 1;
            c += 1;
        }
        c = c - 32;

        let mut i = 0;
        while i < (32 - c + 1) {
            r <<= 1;

            if t >= d {
                r += 1;
                t = t.wrapping_sub(d);
            }
            let lsb = n & 0x1;
            t = (t << 1) + lsb;
            n >>= 1;
            i += 1
        }

        let inv = r;

        SQ1_31(neg * (inv >> 1) as i32)
    }
}

impl From<SQ8_8> for SQ8_0 {
    fn from(f: SQ8_8) -> Self {
        SQ8_0((f.0 >> 8) as i8)
    }
}
impl From<SQ8_8> for SQ1_7 {
    fn from(f: SQ8_8) -> Self {
        SQ1_7((f.0.wrapping_shl(9).wrapping_shr(9)) as i8)
    }
}

impl Mul for SQ1_7 {
    type Output = SQ1_15;
    fn mul(self, rhs: Self) -> SQ1_15 {
        SQ1_15(((self.0 as i16) * (rhs.0 as i16)) << 1)
    }
}

impl Mul for SQ1_15 {
    type Output = SQ1_31;
    fn mul(self, rhs: Self) -> SQ1_31 {
        SQ1_31(((self.0 as i32) * (rhs.0 as i32)) << 1)
    }
}

impl Mul<SQ16_0> for SQ1_15 {
    type Output = SQ16_15;
    fn mul(self, rhs: SQ16_0) -> SQ16_15 {
        SQ16_15(((self.0 as i32) * (rhs.0 as i32)) << 1)
    }
}

impl Mul<SQ8_0> for SQ1_7 {
    type Output = SQ8_8;
    fn mul(self, rhs: SQ8_0) -> SQ8_8 {
        SQ8_8(((self.0 as i16) * (rhs.0 as i16)) << 1)
    }
}

impl Mul for SQ8_0 {
    type Output = SQ16_0;
    fn mul(self, rhs: Self) -> SQ16_0 {
        SQ16_0((self.0 as i16) * (rhs.0 as i16))
    }
}

impl From<SQ32_0> for i32 {
    fn from(n: SQ32_0) -> Self {
        n.0
    }
}

impl From<SQ32_0> for u32 {
    fn from(n: SQ32_0) -> Self {
        n.0 as u32
    }
}

impl Mul for SQ32_0 {
    // Blah
    type Output = SQ32_0;
    fn mul(self, rhs: Self) -> SQ32_0 {
        SQ32_0(self.0 * rhs.0)
    }
}

impl MulAssign for SQ32_0 {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0
    }
}

impl Mul<SQ1_7> for SQ8_0 {
    type Output = SQ8_8;
    fn mul(self, rhs: SQ1_7) -> SQ8_8 {
        SQ8_8(((self.0 as i16) * (rhs.0 as i16)) << 1)
    }
}
impl Mul<SQ1_7> for SQ16_0 {
    type Output = SQ16_0;
    fn mul(self, rhs: SQ1_7) -> SQ16_0 {
        SQ16_0((((self.0 as i32) * (rhs.0 as i32)) << 1) as i16)
    }
}

impl fmt::Display for SQ1_7 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut accum = 0f64;
        let mut j = self.0;
        for i in 0..8 {
            if j < 0 {
                if i == 0 {
                    accum -= 1.;
                } else {
                    accum += (2f64).powf(-i as f64);
                }
            }
            j <<= 1;
        }
        write!(f, "{:8.7}", accum)
    }
}
impl fmt::Display for SQ8_0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl fmt::Display for SQ8_8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut accum = 0f64;
        let mut j = self.0;
        for i in 0..16 {
            if j < 0 {
                if i == 0 {
                    accum -= 128.;
                } else {
                    accum += (2f64).powf(7. - i as f64);
                }
            }
            j <<= 1;
        }
        write!(f, "{:15.7}", accum)
    }
}
impl fmt::Debug for SQ1_7 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:08b} {:x})", self.0, self.0)
    }
}

impl fmt::Debug for SQ8_0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:08b} {:x})", self.0, self.0)
    }
}
impl fmt::Debug for SQ16_0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:016b} {:x})", self.0, self.0)
    }
}
impl fmt::Display for SQ16_0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for SQ8_8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:016b} {:x})", self.0, self.0)
    }
}

impl fmt::Debug for SQ1_15 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:016b} {:x})", self.0, self.0)
    }
}
impl fmt::Display for SQ1_15 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut accum = 0.;
        let mut j = self.0;
        for i in 0..16 {
            if j < 0 {
                if i == 0 {
                    accum -= 1.;
                } else {
                    accum += (2f64).powf(-i as f64);
                }
            }
            j <<= 1;
        }
        write!(f, "{:8.7}", accum)
    }
}

impl fmt::Debug for SQ1_31 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:032b} {:x})", self.0, self.0)
    }
}
impl fmt::Display for SQ1_31 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut accum = 0.;
        let mut j = self.0;
        for i in 0..32 {
            if j < 0 {
                if i == 0 {
                    accum -= 1.;
                } else {
                    accum += (2f64).powf(-i as f64);
                }
            }
            j <<= 1;
        }
        write!(f, "{:8.7}", accum)
    }
}

impl fmt::Debug for SQ16_15 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:032b} {:x})", self.0, self.0)
    }
}
impl fmt::Display for SQ16_15 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut accum = 0.;
        let mut j = self.0;
        let offset = 15;
        for i in 0..32 {
            if j < 0 {
                if i == 0 {
                    accum -= (2f64).powf(offset as f64);
                } else {
                    accum += (2f64).powf((offset - i) as f64);
                }
            }
            j <<= 1;
        }
        write!(f, "{:8.7}", accum)
    }
}
