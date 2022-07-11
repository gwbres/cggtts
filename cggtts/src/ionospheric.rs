/// Klobuchar coefficients
pub type Klobuchar = (f64, f64, f64, f64);

fn deg2rad (deg: f64) -> f64 {
    std::f64::consts::PI * deg / 180.0
}

fn sum (data: f64, coeff: Klobuchar) -> f64 {
    coeff.0 
    + coeff.1 * data 
    + coeff.2 * data.powf(2.0_f64)
    + coeff.3 * data.powf(3.0_f64)
}

/// Computes Ionospheric Delay from Klobuchar Parameters
/// and current space vehicule parameters
/// Inputs:
///   - pos: vehicule (latitude, longitude) estimates in ddeg
///   - e: vehicule elevation in degrees
///   - t: GPS time (seconds)
///   - azim: vehicule azimuth in degrees 
///   - klobuchar: (alpha, beta) coefficients tuple
pub fn klobuchar2delay (pos: (f64,f64), e: f64, azim: f64, t: f64, klobuchar: (Klobuchar, Klobuchar)) -> f64 {
    let e = deg2rad(e);
    let azim = deg2rad(azim);
    let (phi_u, lambd_u) = pos;
    let (alpha, beta) = klobuchar;
    let psi = 0.0137/(e+0.11) - 0.022; // earth centered angle
    
    let mut phi_i = phi_u + psi * azim.cos(); 
    if phi_i < -0.416 {
        phi_i = -0.416;
    } else if phi_i > 0.416 {
        phi_i = 0.416;
    }
    
    let lambd_i = lambd_u + (psi *azim.sin())/phi_i.cos();
    let phi_m = phi_i + 0.064 * (lambd_i - 1.617).cos();
    
    let mut t = 43200.0 * lambd_i + t;
    if t < 0.0 {
        t += 86400.0
    } else if t >= 86400.0 {
        t = 86400.0
    }
    
    // <!> ionospheric delay amplitude <!>
    //let mut a_i = sum(phi_m, alpha);
    //if a_i < 0.0 {
    //    a_i = 0.0
    //}
    
    let mut p_i = sum(phi_m, beta);
    if p_i < 72000.0 {
        p_i = 72000.0
    }

    let x_i = 2.0 * std::f64::consts::PI * (t - 50400.0) / p_i;
    let f = 1.0 + 16.0*(0.53 - e).powf(3.0_f64);

    if x_i.abs() < 1.57 {
        5.0E-9 + sum(phi_m, alpha) * (1.0 - x_i.powf(2.0_f64)/2.0 + x_i.powf(4.0_f64)/24.0) *f 
    } else {
        5.0E-9 * f
    }
    
    //I = fL1/f * I pour translation
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
pub struct IonosphericData {
    /// Measured ionospheric delay 
    /// corresponding to the solution E in section 2.3.3.
    pub msio: f64, 
    /// Slope of the measured ionospheric delay 
    /// corresponding to the solution E in section 2.3.3.
    pub smsi: f64, 
    /// Root-mean-square of the residuals 
    /// corresponding to the solution E in section2.3.3
    pub isg: f64, 
}

impl Into<(f64,f64,f64)> for IonosphericData {
    fn into (self) -> (f64,f64,f64) {
        (self.msio, self.smsi, self.isg)
    }
}

impl From<(f64,f64,f64)> for IonosphericData {
    fn from (data: (f64,f64,f64)) -> Self {
        Self {
            msio: data.0,
            smsi: data.1,
            isg: data.2,
        }
    }
}

impl Default for IonosphericData {
    /// Builds Null Ionospheric Parameter estimates
    fn default() -> Self {
        Self {
            msio: 0.0,
            smsi: 0.0,
            isg: 0.0,
        }
    }
}
