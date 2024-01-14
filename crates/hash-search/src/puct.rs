use crate::tree::TreeNodeMetadata;

pub(crate) fn puct(metadata: &TreeNodeMetadata, exploration_rate: f32) -> f32 {
    exploration_rate * metadata.probability * metadata.value_sum.sqrt()
        / (1 + metadata.visits) as f32
}
