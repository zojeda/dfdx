use super::utils::move_tape_and_add_backward_op;
use crate::prelude::*;

/// `t.sum(-1)`. Reduces the last dimension of the tensor by summing all the values in that dimension.
/// Result [Tensor] has smaller number of dimensions.
///
/// Examples:
/// ```rust
/// # use dfdx::prelude::*;
/// let t = Tensor2D::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
/// let r: Tensor1D<2> = sum_last_dim(t);
/// assert_eq!(r.data(), &[6.0, 15.0]);
/// ```
pub fn sum_last_dim<T: Tensor<Dtype = f32>>(t: T) -> T::LastDimReduced {
    let result = <T::LastDimReduced as Tensor>::NoTape::new_boxed(T::Device::reduce_last_dim(
        t.data(),
        &mut |a, b| a + b,
    ));
    move_tape_and_add_backward_op(t, result, move |t, result, grads| {
        let (t_grad, result_grad) = grads.mut_and_ref(&t, &result);
        T::Device::badd(t_grad, Broadcast(result_grad));
    })
}

macro_rules! sum_last_impl {
    ($typename:ident, [$($Vs:tt),*]) => {
impl<$(const $Vs: usize, )* H: Tape> $typename<$($Vs, )* H> {
    /// Calls [sum_last_dim()] on `self`.
    pub fn sum_last_dim(self) -> <Self as Tensor>::LastDimReduced {
        sum_last_dim(self)
    }
}
    };
}

sum_last_impl!(Tensor0D, []);
sum_last_impl!(Tensor1D, [M]);
sum_last_impl!(Tensor2D, [M, N]);
sum_last_impl!(Tensor3D, [M, N, O]);
sum_last_impl!(Tensor4D, [M, N, O, P]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_last_0d() {
        let t = Tensor0D::new(2.0);
        let r: Tensor0D<OwnedTape> = t.trace().sum_last_dim();
        assert_eq!(r.data(), &2.0);
        let gradients = r.mean().backward();
        assert_eq!(gradients.ref_gradient(&t), &1.0);
    }

    #[test]
    fn test_sum_last_1d() {
        let t: Tensor1D<3> = Tensor1D::new([1.0, 2.0, 3.0]);
        let r: Tensor0D<OwnedTape> = t.trace().sum_last_dim();
        assert_eq!(r.data(), &6.0);
        // NOTE: .exp() so we make sure its using result grad properly
        let gradients = r.exp().mean().backward();
        assert_eq!(gradients.ref_gradient(&t), &[403.4288; 3]);
    }

    #[test]
    fn test_sum_last_2d() {
        let t: Tensor2D<2, 3> = Tensor2D::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        let r: Tensor1D<2, OwnedTape> = t.trace().sum_last_dim();
        assert_eq!(r.data(), &[6.0, 15.0]);
        let gradients = r.mean().backward();
        assert_eq!(
            gradients.ref_gradient(&t),
            &[[0.5, 0.5, 0.5], [0.5, 0.5, 0.5]]
        );
    }

    #[test]
    fn test_sum_last_3d() {
        let t: Tensor3D<4, 2, 3> = Tensor3D::new([
            [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
            [[-1.0, -2.0, -3.0], [-4.0, -5.0, -6.0]],
            [[-3.0, 2.0, -1.0], [-6.0, 5.0, -4.0]],
            [[1.0, -2.0, 3.0], [4.0, -5.0, 6.0]],
        ]);
        let r: Tensor2D<4, 2, OwnedTape> = t.trace().sum_last_dim();
        assert_eq!(
            r.data(),
            &[[6.0, 15.0], [-6.0, -15.0], [-2.0, -5.0], [2.0, 5.0],]
        );
        let gradients = r.mean().backward();
        assert_eq!(gradients.ref_gradient(&t), &[[[1.0 / 8.0; 3]; 2]; 4]);
    }
}
