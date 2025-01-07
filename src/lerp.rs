use std::ops::{Add, Mul};

#[derive(Debug)]
pub struct MonomialLerp {
    m_degree: f32,
    m_timestep: f32,
    m_at_boundary: bool
}

// Pretty beefy iterator I won't lie
pub struct LerpIntoIter<T: Add + Mul<f32, Output = T> + Clone> {
    lerper: MonomialLerp,
    start: T,
    end: T,
    point_at: usize,
    num_points: usize,
    step_size: f32
}

impl MonomialLerp {
    /** runtime check for whether a given timestep is within the inclusive range [0, 1] */
    fn timestep_check(timestep: f32) -> () {
        assert!(
            timestep < 0f32 || timestep > 1f32,
            "MonomialLerp::timestep must be a f32 within the inclusive range [0, 1], you entered {}",
            timestep
        )
    }

    // runtime check for whether a given degree is greater than 0
    fn degree_check(degree: f32) -> () {
        assert!(
            degree > 0.0,
            "MonomialLerp::degree must be a f32 greater than 0.0, you entered {}",
            degree
        );
    }

    // constructs a new MonommialLerp structure of a given degree. Timestep is set to 0 by default
    pub fn new(degree: f32) -> Self {
        MonomialLerp::degree_check(degree);

        return MonomialLerp {
            m_degree: degree,
            m_timestep: 0f32,
            m_at_boundary: false
        };
    }

    pub unsafe fn new_unchecked(degree: f32) -> Self {
        return MonomialLerp {
            m_degree: degree,
            m_timestep: 0f32,
            m_at_boundary: false
        };
    }

    // constructs a new MonomialLerp structure of a given degree starting at a given timestep
    pub fn new_at_timestep(degree: f32, timestep: f32) -> Self {
        MonomialLerp::timestep_check(timestep);
        MonomialLerp::degree_check(degree);

        return MonomialLerp {
            m_degree: degree,
            m_timestep: timestep,
            m_at_boundary: false
        };
    }

    pub unsafe fn new_at_timestep_unchecked(degree: f32, timestep: f32) -> Self {
        return MonomialLerp {
            m_degree: degree,
            m_timestep: timestep,
            m_at_boundary: false
        };
    }

    // constructs a new MonomialLerp structure of a given degree starting at a timestep determinined by 1 / (num_steps - 1)
    pub fn new_from_steps(degree: f32, num_steps: u32) -> Self {
        MonomialLerp::degree_check(degree);

        let parition: f32 = 1f32 / f32::abs((num_steps - 1) as f32);
        return MonomialLerp {
            m_degree: degree,
            m_timestep: parition,
            m_at_boundary: false
        };
    }

    pub unsafe fn new_from_steps_unchecked(degree: f32, num_steps: u32) -> Self {
        let parition: f32 = 1f32 / f32::abs((num_steps - 1) as f32);
        return MonomialLerp {
            m_degree: degree,
            m_timestep: parition,
            m_at_boundary: false
        };
    }

    pub fn reset(&mut self) -> &mut Self {
        self.m_timestep = 0f32;
        return self;
    }

    pub fn reset_at(&mut self, timestep: f32) -> &mut Self {
        MonomialLerp::timestep_check(timestep);
        self.m_timestep = timestep;
        return self;
    }

    pub fn get_step(&self) -> f32 {
        return self.m_timestep;
    }

    pub fn get_degree(&self) -> f32 {
        return self.m_degree;
    }

    pub fn at_boundary(&self) -> bool {
        return self.m_at_boundary;
    }

    pub fn compute<T: Add + Mul<f32, Output = T> + Clone>(start: &T, end: &T, timestep: f32, degree: f32) -> <T as Add>::Output {
        MonomialLerp::timestep_check(timestep);
        MonomialLerp::degree_check(degree);

        let monomial_pow: f32 = f32::powf(timestep, degree);
        return start.clone() * (1f32 - monomial_pow) + end.clone() * monomial_pow;
    }

    pub unsafe fn compute_unchecked<T: Add + Mul<f32, Output = T> + Clone>(start: &T, end: &T, timestep: f32, degree: f32) -> <T as Add>::Output {
        let monomial_pow: f32 = f32::powf(timestep, degree);
        return start.clone() * (1f32 - monomial_pow) + end.clone() * monomial_pow;
    }

    pub fn step<T: Add + Mul<f32,Output=T> + Clone>(&mut self, start: &T, end: &T) -> <T as Add>::Output {
        let monomial_pow: f32 = f32::powf(self.m_timestep, self.m_degree);
        let result: <T as Add>::Output = start.clone() * (1f32 - monomial_pow) + end.clone() * monomial_pow;

        self.m_timestep += 0.1f32;
        self.m_at_boundary = self.m_timestep > 1.0f32 || self.m_timestep < 0.0f32;
        self.m_timestep = f32::max(0f32, f32::min(1f32, self.m_timestep));

        return result;
    }

    pub fn step_by<T: Add + Mul<f32,Output=T> + Clone>(&mut self, start: &T, end: &T, by: f32) -> <T as Add>::Output {
        let monomial_pow: f32 = f32::powf(self.m_timestep, self.m_degree);
        let result: <T as Add>::Output = start.clone() * (1f32 - monomial_pow) + end.clone() * monomial_pow;

        self.m_timestep += by;
        self.m_at_boundary = self.m_timestep > 1.0f32 || self.m_timestep < 0.0f32;
        self.m_timestep = f32::max(0f32, f32::min(1f32, self.m_timestep));

        return result;
    }

    pub fn into_iter<T: Add + Clone + Mul<f32, Output = T>>(self, start: &T, end: &T, num_steps: usize) -> LerpIntoIter<T> {
        return LerpIntoIter {
            lerper: self,
            start: start.clone(),
            end: end.clone(),
            point_at: 0,
            num_points: num_steps,
            step_size: 1f32 / f32::abs((num_steps - 1) as f32)
        };
    }

    pub fn quick_iter<T: Add + Clone + Mul<f32, Output = T>>(degree: f32, start: &T, end: &T, num_steps: usize) -> LerpIntoIter<T> {
        return LerpIntoIter {
            lerper: MonomialLerp {
                m_degree: degree,
                m_timestep: 0f32,
                m_at_boundary: false
            },
            start: start.clone(),
            end:  end.clone(),
            point_at: 0,
            num_points: num_steps,
            step_size: 1f32 / f32::abs((num_steps) as f32)
        };
    }

}

impl Drop for MonomialLerp {
    fn drop(&mut self) {
        // nothing lmaooooo
    }
}

impl<T: Add + Clone + Mul<f32, Output = T>> Iterator for LerpIntoIter<T> {
    type Item = <T as Add>::Output;
    fn next(&mut self) -> std::option::Option<<T as Add>::Output> {
        if self.point_at < self.num_points {
            self.point_at += 1;
            return Some(self.lerper.step_by(&self.start, &self.end, self.step_size));
        }
        
        // } else if self.point_at == self.num_points {
        //     return Some(MonomialLerp::compute(&self.end, &self.end, 0f32, self.lerper.m_degree));
        // }

        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::MonomialLerp;


  #[test]
  fn lerp_into_iter_test() {
    let start: f32 = 0f32;
    let end: f32 = 100f32;

    for (i, intermediate) in MonomialLerp::new(2f32).into_iter(&start,&end, 100).enumerate() {
        println!("index: {}, value: {}", i, intermediate);
    }
  }

  #[test]
  fn quick_iter_test() {
    for portion in MonomialLerp::quick_iter(1f32, &0f32, &1f32, 10) {
        println!("{}", portion);
    }
  }
}