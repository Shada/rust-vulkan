

pub unsafe fn update_model_matrix()
{
    let time = 0.0f32;
    let model = glm::rotate(
        &glm::identity(),
        time * glm::radians(&glm::vec1(90.0))[0],
        &glm::vec3(0.0,0.0,1.0),
    );
    let (_, model_bytes, _) = model.as_slice().align_to::<u8>();


}