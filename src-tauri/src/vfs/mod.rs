//! Unified data layer: a generic folder tree (in the main app db, see
//! [`folders`]) over content stored in a sidecar `agentflare-store` database
//! (`vfs.db`, opened here) for anything that doesn't fit the curriculum
//! schema -- e.g. a document a student or teacher imports later.
//!
//! Scoped deliberately narrow: just folders + document storage/search.
//! Nothing here does RAG chunking, embeddings, or multimodal indexing --
//! none of that has a use in this app yet. See docs/unified-data-layer-spec.md.

pub mod folders;

use crate::error::AppError;
use crate::models::{VfsDocument, VfsDocumentMatch};
use agentflare_store::Store;
use std::path::Path;

/// A single fixed project id: this app has no multi-project concept, so
/// `agentflare-store`'s `project_id` partitioning is unused here.
const PROJECT_ID: &str = "sarvo-shala";

pub struct Vfs {
    store: Store,
}

impl Vfs {
    pub fn open_or_init(path: &Path) -> Result<Vfs, AppError> {
        let store = Store::open_file(path)?;
        Ok(Vfs { store })
    }

    /// Create or overwrite the document at `path` with `content`.
    pub fn put_document(
        &self,
        path: &str,
        title: &str,
        content: &str,
    ) -> Result<VfsDocument, AppError> {
        let opts = agentflare_store::documents::DocUpsertOpts {
            title: Some(title.to_string()),
            ..Default::default()
        };
        let doc = self
            .store
            .doc_upsert_with_opts(PROJECT_ID, path, content, opts)?;
        Ok(doc.into())
    }

    pub fn get_document(&self, id: &str) -> Result<Option<VfsDocument>, AppError> {
        Ok(self.store.doc_get(id)?.map(Into::into))
    }

    pub fn delete_document(&self, id: &str) -> Result<bool, AppError> {
        Ok(self.store.doc_delete(id)?)
    }

    pub fn search_documents(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<VfsDocumentMatch>, AppError> {
        let matches = self.store.doc_search(PROJECT_ID, query, limit)?;
        Ok(matches.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_vfs() -> (Vfs, std::path::PathBuf) {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("shala_vfs_{n}.db"));
        let _ = std::fs::remove_file(&p);
        let vfs = Vfs::open_or_init(&p).unwrap();
        (vfs, p)
    }

    #[test]
    fn put_get_and_delete_document_roundtrip() {
        let (vfs, p) = temp_vfs();
        let doc = vfs
            .put_document(
                "notes/photosynthesis.md",
                "Photosynthesis",
                "light + CO2 -> glucose",
            )
            .unwrap();
        assert_eq!(doc.title, "Photosynthesis");

        let fetched = vfs.get_document(&doc.id).unwrap();
        assert_eq!(fetched.map(|d| d.id), Some(doc.id.clone()));

        assert!(vfs.delete_document(&doc.id).unwrap());
        assert!(vfs.get_document(&doc.id).unwrap().is_none());
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn search_finds_matching_document_content() {
        let (vfs, p) = temp_vfs();
        vfs.put_document(
            "notes/a.md",
            "A",
            "mitochondria is the powerhouse of the cell",
        )
        .unwrap();
        vfs.put_document("notes/b.md", "B", "unrelated content about rivers")
            .unwrap();

        let hits = vfs.search_documents("mitochondria", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, "notes/a.md");
        let _ = std::fs::remove_file(&p);
    }
}
