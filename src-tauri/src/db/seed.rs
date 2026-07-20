use crate::db::Db;
use crate::error::AppError;

struct SubjectSeed {
    id: i64,
    grade: i64,
    name: &'static str,
    name_hi: &'static str,
}

struct ChapterSeed {
    id: i64,
    subject_id: i64,
    ord: i64,
    title: &'static str,
    title_hi: &'static str,
    summary: &'static str,
}

struct ConceptSeed {
    id: i64,
    chapter_id: i64,
    ord: i64,
    name: &'static str,
    key_formula: &'static str,
    common_error: &'static str,
    exam_weight: i64,
}

struct SnippetSeed {
    id: i64,
    chapter_id: i64,
    concept_id: i64,
    ord: i64,
    body: &'static str,
}

struct QSeed {
    id: i64,
    concept_id: i64,
    difficulty_layer: i64,
    bloom: &'static str,
    prompt: &'static str,
    trap: &'static str,
    explanation: &'static str,
    opts: [&'static str; 4],
    correct: usize,
}

struct PrereqSeed {
    concept_id: i64,
    prereq_id: i64,
}

const SUBJECTS: &[SubjectSeed] = &[SubjectSeed {
    id: 1,
    grade: 8,
    name: "Science",
    name_hi: "विज्ञान",
}];

const CHAPTERS: &[ChapterSeed] = &[
    ChapterSeed {
        id: 1,
        subject_id: 1,
        ord: 1,
        title: "Force and Pressure",
        title_hi: "बल और दाब",
        summary:
            "How pushes and pulls change motion, and how force spread over area becomes pressure.",
    },
    ChapterSeed {
        id: 2,
        subject_id: 1,
        ord: 2,
        title: "Friction",
        title_hi: "घर्षण",
        summary: "The contact force that opposes motion, what affects it, and how we control it.",
    },
    ChapterSeed {
        id: 3,
        subject_id: 1,
        ord: 3,
        title: "Cell Structure",
        title_hi: "कोशिका संरचना",
        summary: "The basic unit of life, how plant and animal cells differ, and key organelles.",
    },
];

const CONCEPTS: &[ConceptSeed] = &[
    ConceptSeed {
        id: 1,
        chapter_id: 1,
        ord: 1,
        name: "Force basics",
        key_formula: "F causes change in state of motion",
        common_error: "Confusing mass with weight",
        exam_weight: 3,
    },
    ConceptSeed {
        id: 2,
        chapter_id: 1,
        ord: 2,
        name: "Pressure",
        key_formula: "P = F/A",
        common_error: "Forgetting area is in the denominator",
        exam_weight: 5,
    },
    ConceptSeed {
        id: 3,
        chapter_id: 1,
        ord: 3,
        name: "Atmospheric pressure",
        key_formula: "",
        common_error: "",
        exam_weight: 2,
    },
    ConceptSeed {
        id: 4,
        chapter_id: 2,
        ord: 1,
        name: "Types of friction",
        key_formula: "",
        common_error: "Thinking rolling friction > sliding friction",
        exam_weight: 3,
    },
    ConceptSeed {
        id: 5,
        chapter_id: 2,
        ord: 2,
        name: "Factors affecting friction",
        key_formula: "",
        common_error: "",
        exam_weight: 3,
    },
    ConceptSeed {
        id: 6,
        chapter_id: 2,
        ord: 3,
        name: "Reducing friction",
        key_formula: "",
        common_error: "",
        exam_weight: 2,
    },
    ConceptSeed {
        id: 7,
        chapter_id: 3,
        ord: 1,
        name: "Cell discovery",
        key_formula: "",
        common_error: "",
        exam_weight: 1,
    },
    ConceptSeed {
        id: 8,
        chapter_id: 3,
        ord: 2,
        name: "Plant vs animal cells",
        key_formula: "",
        common_error: "Assuming all cells have cell walls",
        exam_weight: 4,
    },
    ConceptSeed {
        id: 9,
        chapter_id: 3,
        ord: 3,
        name: "Cell organelles",
        key_formula: "",
        common_error: "",
        exam_weight: 4,
    },
];

const SNIPPETS: &[SnippetSeed] = &[
    SnippetSeed { id: 1, chapter_id: 1, concept_id: 1, ord: 1, body: "A force is a push or a pull that acts on an object. Forces can make a resting object move, slow down a moving object, or change the direction in which it travels. They can also change the shape of soft objects such as clay." },
    SnippetSeed { id: 2, chapter_id: 1, concept_id: 1, ord: 2, body: "Mass is the amount of matter in an object and stays the same anywhere, while weight is the pull of gravity on that mass and changes with location. A book on Earth and the same book on the Moon have equal mass but different weight." },
    SnippetSeed { id: 3, chapter_id: 1, concept_id: 2, ord: 1, body: "Pressure tells us how concentrated a force is. It is the force applied divided by the area over which it spreads, written as P = F / A. The same force over a tiny area gives a very large pressure." },
    SnippetSeed { id: 4, chapter_id: 1, concept_id: 2, ord: 2, body: "Because area sits in the denominator, a smaller contact area means higher pressure for the same force. This is why a sharp nail pierces wood easily while a blunt one does not, even when you push with equal strength." },
    SnippetSeed { id: 5, chapter_id: 1, concept_id: 3, ord: 1, body: "The air around us has weight, and the column of air above the ground presses down on everything. This downward push of the atmosphere is called atmospheric pressure." },
    SnippetSeed { id: 6, chapter_id: 1, concept_id: 3, ord: 2, body: "Higher up a mountain there is less air above you, so the atmospheric pressure is lower than at sea level. That is why water boils at a lower temperature on a mountain top." },
    SnippetSeed { id: 7, chapter_id: 2, concept_id: 4, ord: 1, body: "Friction is the force that opposes the relative motion of two surfaces in contact. Without friction we could not walk, because our feet would simply slip against the ground." },
    SnippetSeed { id: 8, chapter_id: 2, concept_id: 4, ord: 2, body: "There are several kinds of friction. Sliding friction acts when surfaces slide past each other, while rolling friction acts when an object rolls. Rolling friction is generally much smaller than sliding friction." },
    SnippetSeed { id: 9, chapter_id: 2, concept_id: 5, ord: 1, body: "Friction depends mainly on the nature of the two surfaces and on the normal force pressing them together. Rough surfaces and heavier normal forces usually produce more friction." },
    SnippetSeed { id: 10, chapter_id: 2, concept_id: 5, ord: 2, body: "The normal force is the perpendicular push between surfaces. Pressing two blocks together harder increases the normal force, which in turn increases the friction between them." },
    SnippetSeed { id: 11, chapter_id: 2, concept_id: 6, ord: 1, body: "We often want less friction so machines run smoothly. A thin oil or grease layer keeps surfaces from directly touching, which greatly lowers the friction between them." },
    SnippetSeed { id: 12, chapter_id: 2, concept_id: 6, ord: 2, body: "Ball bearings reduce friction by turning sliding motion into rolling motion. Because rolling friction is small, the axle can turn with much less resistance than if it slid against its housing." },
    SnippetSeed { id: 13, chapter_id: 3, concept_id: 7, ord: 1, body: "Cells are so small that they were unseen until instruments could magnify tiny things. The first clear views of cells came from early microscopes built in the seventeenth century." },
    SnippetSeed { id: 14, chapter_id: 3, concept_id: 7, ord: 2, body: "Robert Hooke looked at thin slices of cork and saw many empty box-like spaces. He named these compartments 'cells' because they reminded him of the small rooms in a monastery." },
    SnippetSeed { id: 15, chapter_id: 3, concept_id: 8, ord: 1, body: "Plant cells are enclosed by a rigid cell wall outside the membrane, giving them a fixed rectangular shape. Animal cells have no cell wall and are usually rounder and more flexible." },
    SnippetSeed { id: 16, chapter_id: 3, concept_id: 8, ord: 2, body: "Plant cells contain green chloroplasts that capture sunlight for photosynthesis, while animal cells lack chloroplasts. Both kinds of cells still share a membrane, cytoplasm, and a nucleus." },
    SnippetSeed { id: 17, chapter_id: 3, concept_id: 9, ord: 1, body: "The nucleus acts as the control centre of the cell and stores the genetic material, DNA, that carries instructions for the cell's activities. It is usually the largest organelle and sits near the centre." },
    SnippetSeed { id: 18, chapter_id: 3, concept_id: 9, ord: 2, body: "Mitochondria are organelles that release energy the cell can use by breaking down food. Because they supply most of the cell's usable energy, they are often called the powerhouses of the cell." },
];

const QUESTIONS: &[QSeed] = &[
    // Concept 1 — Force basics
    QSeed { id: 1, concept_id: 1, difficulty_layer: 1, bloom: "remember", prompt: "What does a force do to an object?", trap: "Mass and weight are the same quantity.", explanation: "A force is a push or pull that can change an object's state of motion or its shape.", opts: ["Changes its state of motion or shape", "Changes its colour", "Makes it heavier", "Cools it down"], correct: 0 },
    QSeed { id: 2, concept_id: 1, difficulty_layer: 2, bloom: "understand", prompt: "A book rests on a table. Which statement is true?", trap: "Weight and mass are interchangeable terms.", explanation: "The table exerts an upward normal force that balances the book's weight, so it stays at rest.", opts: ["The table pushes up on the book with a normal force", "No forces act on the book", "Gravity only acts when the book falls", "The book pulls the table down only"], correct: 0 },
    QSeed { id: 3, concept_id: 1, difficulty_layer: 3, bloom: "apply", prompt: "A 5 N and a 3 N force act in the same direction on a cart. What is the net force?", trap: "Forces in the same direction add like vectors.", explanation: "Forces in the same direction add, so the net force is 5 + 3 = 8 N.", opts: ["8 N", "2 N", "15 N", "1.67 N"], correct: 0 },
    // Concept 2 — Pressure
    QSeed { id: 4, concept_id: 2, difficulty_layer: 1, bloom: "remember", prompt: "Pressure is defined as...", trap: "Area appears in the numerator when it should be the denominator.", explanation: "Pressure is force divided by the area over which it acts: P = F / A.", opts: ["Force divided by area", "Force multiplied by area", "Mass divided by volume", "Energy per second"], correct: 0 },
    QSeed { id: 5, concept_id: 2, difficulty_layer: 2, bloom: "understand", prompt: "Why does a sharp knife cut better than a blunt one?", trap: "Forgetting area is in the denominator of P = F / A.", explanation: "A sharp edge contacts a smaller area, so for the same force the pressure is much higher.", opts: ["It applies the same force over a smaller area, raising pressure", "It applies more total force", "It is simply made of harder metal", "It reduces the weight of the object"], correct: 0 },
    QSeed { id: 6, concept_id: 2, difficulty_layer: 3, bloom: "apply", prompt: "A 200 N force is applied over 0.1 m². What is the pressure?", trap: "0.1 m² in the denominator gives 2000, not 20.", explanation: "P = F / A = 200 / 0.1 = 2000 pascals.", opts: ["2000 Pa", "20 Pa", "200 Pa", "20000 Pa"], correct: 0 },
    // Concept 3 — Atmospheric pressure
    QSeed { id: 7, concept_id: 3, difficulty_layer: 1, bloom: "remember", prompt: "Atmospheric pressure is caused by...", trap: "", explanation: "The weight of the column of air above us presses down on the surface below.", opts: ["The weight of air above us", "The Earth's magnetic field", "Solar radiation", "Ocean tides"], correct: 0 },
    QSeed { id: 8, concept_id: 3, difficulty_layer: 2, bloom: "understand", prompt: "Why does atmospheric pressure decrease as altitude increases?", trap: "", explanation: "Higher up there is less air above you, so the weight of the air column is smaller.", opts: ["There is less air above you", "The air only gets colder", "Gravity weakens sharply", "Air becomes denser"], correct: 0 },
    QSeed { id: 9, concept_id: 3, difficulty_layer: 3, bloom: "apply", prompt: "A barometer reads lower on a mountaintop than at sea level. This shows...", trap: "", explanation: "Lower readings with height show that atmospheric pressure falls as altitude rises.", opts: ["Pressure falls with altitude", "Pressure rises with altitude", "Pressure is constant", "Altitude has no effect"], correct: 0 },
    // Concept 4 — Types of friction
    QSeed { id: 10, concept_id: 4, difficulty_layer: 1, bloom: "remember", prompt: "Which force opposes motion between surfaces in contact?", trap: "", explanation: "Friction is the contact force that resists relative motion between surfaces.", opts: ["Friction", "Gravity", "Magnetism", "Tension"], correct: 0 },
    QSeed { id: 11, concept_id: 4, difficulty_layer: 2, bloom: "understand", prompt: "Compared with sliding friction, rolling friction is generally...", trap: "Rolling friction is smaller than sliding friction.", explanation: "Wheels roll instead of slide, so rolling friction is usually much less than sliding friction.", opts: ["Less than sliding friction", "Greater than sliding friction", "Equal to sliding friction", "Always zero"], correct: 0 },
    QSeed { id: 12, concept_id: 4, difficulty_layer: 3, bloom: "apply", prompt: "You slide a box, then put it on wheels. Friction...", trap: "", explanation: "Wheels replace sliding with rolling, which lowers the friction resisting motion.", opts: ["Decreases because rolling replaces sliding", "Increases", "Stays the same", "Becomes zero"], correct: 0 },
    // Concept 5 — Factors affecting friction
    QSeed { id: 13, concept_id: 5, difficulty_layer: 1, bloom: "remember", prompt: "Friction mainly depends on...", trap: "", explanation: "Friction depends on the two surface materials and the normal force pressing them together.", opts: ["The nature of surfaces and the normal force", "Only the object's colour", "Only its speed", "Only its mass"], correct: 0 },
    QSeed { id: 14, concept_id: 5, difficulty_layer: 2, bloom: "understand", prompt: "Increasing the normal force between two surfaces usually...", trap: "", explanation: "A larger normal force presses surfaces together more, increasing friction.", opts: ["Increases friction", "Decreases friction", "Removes friction", "Has no effect"], correct: 0 },
    QSeed { id: 15, concept_id: 5, difficulty_layer: 3, bloom: "apply", prompt: "A heavier load is dragged across the floor. Compared with a lighter load, friction is...", trap: "", explanation: "Heavier load means a larger normal force, so friction is greater.", opts: ["Greater because the normal force is larger", "Smaller", "The same", "Zero"], correct: 0 },
    // Concept 6 — Reducing friction
    QSeed { id: 16, concept_id: 6, difficulty_layer: 1, bloom: "remember", prompt: "Which method reduces friction?", trap: "", explanation: "A lubricant such as oil keeps surfaces apart and lowers friction.", opts: ["Using a lubricant like oil", "Increasing surface roughness", "Adding weight", "Heating the surface"], correct: 0 },
    QSeed { id: 17, concept_id: 6, difficulty_layer: 2, bloom: "understand", prompt: "Ball bearings reduce friction by...", trap: "", explanation: "They let surfaces roll past each other instead of sliding, which is much smoother.", opts: ["Replacing sliding with rolling", "Increasing contact area", "Gluing surfaces together", "Only adding grease"], correct: 0 },
    QSeed { id: 18, concept_id: 6, difficulty_layer: 3, bloom: "apply", prompt: "Two identical blocks slide; one has oil between the surfaces. The oiled block...", trap: "", explanation: "Oil acts as a lubricant, so the oiled block experiences less friction.", opts: ["Experiences less friction", "Experiences more friction", "Experiences the same friction", "Stops instantly"], correct: 0 },
    // Concept 7 — Cell discovery
    QSeed { id: 19, concept_id: 7, difficulty_layer: 1, bloom: "remember", prompt: "The cell was first observed using...", trap: "", explanation: "Early microscopes revealed structures too small for the naked eye.", opts: ["A microscope", "A telescope", "A simple magnifying glass alone", "The naked eye"], correct: 0 },
    QSeed { id: 20, concept_id: 7, difficulty_layer: 2, bloom: "understand", prompt: "Robert Hooke coined 'cell' after seeing...", trap: "", explanation: "He saw box-like empty spaces in cork and named them cells.", opts: ["Box-like compartments in cork", "Animal blood", "A leaf's green colour", "Bacteria dividing"], correct: 0 },
    QSeed { id: 21, concept_id: 7, difficulty_layer: 3, bloom: "apply", prompt: "An object too small to see with the naked eye is best studied with...", trap: "", explanation: "A microscope magnifies tiny objects so they become visible.", opts: ["A microscope", "A lens held by hand only", "A ruler", "A mirror"], correct: 0 },
    // Concept 8 — Plant vs animal cells
    QSeed { id: 22, concept_id: 8, difficulty_layer: 1, bloom: "remember", prompt: "Which structure is found in plant cells but usually not in animal cells?", trap: "Animal cells do not have a cell wall.", explanation: "Plant cells have a rigid cell wall outside the membrane; animal cells do not.", opts: ["Cell wall", "Cell membrane", "Cytoplasm", "Nucleus"], correct: 0 },
    QSeed { id: 23, concept_id: 8, difficulty_layer: 2, bloom: "understand", prompt: "Plant cells often contain chloroplasts because they...", trap: "", explanation: "Chloroplasts capture sunlight to carry out photosynthesis and make food.", opts: ["Carry out photosynthesis", "Produce proteins only", "Store DNA", "Divide faster"], correct: 0 },
    QSeed { id: 24, concept_id: 8, difficulty_layer: 3, bloom: "apply", prompt: "A cell with no cell wall and no chloroplasts is most likely...", trap: "", explanation: "Lack of a wall and chloroplasts points to an animal cell.", opts: ["An animal cell", "A plant cell", "A bacterial cell", "A fungal cell with a wall"], correct: 0 },
    // Concept 9 — Cell organelles
    QSeed { id: 25, concept_id: 9, difficulty_layer: 1, bloom: "remember", prompt: "The nucleus is the organelle that...", trap: "", explanation: "The nucleus stores the cell's DNA and controls its activities.", opts: ["Stores genetic material (DNA)", "Produces energy", "Packages proteins for export", "Controls water balance"], correct: 0 },
    QSeed { id: 26, concept_id: 9, difficulty_layer: 2, bloom: "understand", prompt: "Mitochondria are called the powerhouse because they...", trap: "", explanation: "They release most of the usable energy (ATP) the cell needs.", opts: ["Produce most of the cell's usable energy", "Store waste", "Make the cell wall", "Carry out photosynthesis"], correct: 0 },
    QSeed { id: 27, concept_id: 9, difficulty_layer: 3, bloom: "apply", prompt: "A cell needing lots of energy, like a muscle cell, would have many...", trap: "", explanation: "Energy-hungry cells contain many mitochondria to supply ATP.", opts: ["Mitochondria", "Vacuoles", "Cell walls", "Chloroplasts"], correct: 0 },
];

const PREREQS: &[PrereqSeed] = &[
    PrereqSeed {
        concept_id: 2,
        prereq_id: 1,
    },
    PrereqSeed {
        concept_id: 5,
        prereq_id: 4,
    },
    PrereqSeed {
        concept_id: 9,
        prereq_id: 8,
    },
];

pub fn seed_if_empty(db: &Db) -> Result<(), AppError> {
    let has: i64 = db.with(|c| c.query_row("SELECT count(*) FROM subjects", [], |r| r.get(0)))?;
    if has > 0 {
        return Ok(());
    }

    db.with(|c| {
        let tx = c.transaction()?;
        for s in SUBJECTS {
            tx.execute(
                "INSERT INTO subjects (id, grade, name, name_hi) VALUES (?1,?2,?3,?4)",
                (s.id, s.grade, s.name, s.name_hi),
            )?;
        }
        for ch in CHAPTERS {
            tx.execute(
                "INSERT INTO chapters (id, subject_id, ord, title, title_hi, summary) VALUES (?1,?2,?3,?4,?5,?6)",
                (ch.id, ch.subject_id, ch.ord, ch.title, ch.title_hi, ch.summary),
            )?;
        }
        for cn in CONCEPTS {
            tx.execute(
                "INSERT INTO concepts (id, chapter_id, ord, name, key_formula, common_error, exam_weight) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                (cn.id, cn.chapter_id, cn.ord, cn.name, cn.key_formula, cn.common_error, cn.exam_weight),
            )?;
        }
        for sn in SNIPPETS {
            tx.execute(
                "INSERT INTO chapter_text (id, chapter_id, concept_id, ord, body) VALUES (?1,?2,?3,?4,?5)",
                (sn.id, sn.chapter_id, sn.concept_id, sn.ord, sn.body),
            )?;
        }
        for q in QUESTIONS {
            tx.execute(
                "INSERT INTO questions (id, concept_id, qtype, bloom_level, difficulty_layer, prompt, trap, explanation) VALUES (?1,?2,'mcq',?3,?4,?5,?6,?7)",
                (q.id, q.concept_id, q.bloom, q.difficulty_layer, q.prompt, q.trap, q.explanation),
            )?;
            let mut pairs: [(&str, bool); 4] = [
                (q.opts[0], 0 == q.correct),
                (q.opts[1], 1 == q.correct),
                (q.opts[2], 2 == q.correct),
                (q.opts[3], 3 == q.correct),
            ];
            shuffle4(&mut pairs, q.id);
            for (ord, (body, is_correct)) in pairs.iter().enumerate() {
                tx.execute(
                    "INSERT INTO question_options (question_id, ord, body, is_correct) VALUES (?1,?2,?3,?4)",
                    (q.id, ord as i64 + 1, *body, if *is_correct { 1 } else { 0 }),
                )?;
            }
        }
        for p in PREREQS {
            tx.execute(
                "INSERT INTO concept_prereqs (concept_id, prereq_id) VALUES (?1,?2)",
                (p.concept_id, p.prereq_id),
            )?;
        }
        // Seed initial mastery so progress is meaningful before any quiz.
        let now = now_secs();
        for cn in CONCEPTS {
            tx.execute(
                "INSERT OR IGNORE INTO concept_mastery (concept_id, p_known, attempts, correct, updated_at) VALUES (?1,0.2,0,0,?2)",
                (cn.id, now),
            )?;
        }
        tx.commit()?;
        Ok::<(), rusqlite::Error>(())
    })?;

    Ok(())
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Deterministic Fisher-Yates permutation of 4 items seeded by `seed`.
fn shuffle4<T>(arr: &mut [T; 4], seed: i64) {
    let mut s: u64 = seed.unsigned_abs() | 1;
    for i in (1..4).rev() {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = ((s >> 33) % (i as u64 + 1)) as usize;
        arr.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> (Db, std::path::PathBuf) {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("shala_seed_{}.db", n));
        let _ = std::fs::remove_file(&p);
        let db = Db::open_or_init(&p).unwrap();
        (db, p)
    }

    #[test]
    fn seed_populates() {
        let (db, p) = temp_db();
        seed_if_empty(&db).unwrap();
        let concepts: i64 = db
            .with(|c| c.query_row("SELECT count(*) FROM concepts", [], |r| r.get(0)))
            .unwrap();
        let questions: i64 = db
            .with(|c| c.query_row("SELECT count(*) FROM questions", [], |r| r.get(0)))
            .unwrap();
        assert_eq!(concepts, 9);
        assert_eq!(questions, 27);
        // every question has exactly one correct option
        let bad: i64 = db
            .with(|c| {
                c.query_row(
                    "SELECT count(*) FROM (
                       SELECT question_id, count(*) c
                       FROM question_options WHERE is_correct=1
                       GROUP BY question_id HAVING c <> 1
                     )",
                    [],
                    |r| r.get(0),
                )
            })
            .unwrap();
        assert_eq!(bad, 0);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn seed_twice_no_dup() {
        let (db, p) = temp_db();
        seed_if_empty(&db).unwrap();
        seed_if_empty(&db).unwrap();
        let concepts: i64 = db
            .with(|c| c.query_row("SELECT count(*) FROM concepts", [], |r| r.get(0)))
            .unwrap();
        let questions: i64 = db
            .with(|c| c.query_row("SELECT count(*) FROM questions", [], |r| r.get(0)))
            .unwrap();
        assert_eq!(concepts, 9);
        assert_eq!(questions, 27);
        let _ = std::fs::remove_file(&p);
    }
}
