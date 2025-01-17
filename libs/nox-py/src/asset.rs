use crate::*;

#[pyclass]
#[derive(Clone)]
pub struct PyBufBytes {
    pub bytes: bytes::Bytes,
}

pub struct PyAsset {
    pub object: PyObject,
}

impl PyAsset {
    pub fn try_new(py: Python<'_>, object: PyObject) -> Result<Self, Error> {
        let _ = object.getattr(py, "asset_id")?;
        let _ = object.getattr(py, "bytes")?;
        Ok(Self { object })
    }
}

impl PyAsset {
    pub fn asset_id(&self) -> conduit::AssetId {
        Python::with_gil(|py| {
            let id: u64 = self
                .object
                .call_method0(py, "asset_id")
                .unwrap()
                .extract(py)
                .unwrap();
            conduit::AssetId(id)
        })
    }

    pub fn bytes(&self) -> Result<bytes::Bytes, Error> {
        Python::with_gil(|py| {
            let bytes: PyBufBytes = self.object.call_method0(py, "bytes")?.extract(py)?;
            Ok(bytes.bytes)
        })
    }
}

#[derive(Clone)]
#[pyclass]
pub struct Handle {
    pub inner: nox_ecs::Handle<()>,
}

#[pymethods]
impl Handle {
    pub fn asarray(&self) -> Result<PyObject, Error> {
        Ok(nox::NoxprScalarExt::constant(self.inner.id).to_jax()?)
    }

    pub fn flatten(&self) -> Result<((PyObject,), Option<()>), Error> {
        let jax = nox::NoxprScalarExt::constant(self.inner.id).to_jax()?;
        Ok(((jax,), None))
    }

    #[staticmethod]
    fn unflatten(_aux: PyObject, _jax: PyObject) -> Self {
        todo!()
    }

    #[staticmethod]
    fn from_array(_arr: PyObject) -> Self {
        todo!()
    }
}
