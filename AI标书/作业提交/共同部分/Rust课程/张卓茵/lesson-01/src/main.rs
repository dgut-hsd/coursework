#[derive(Debug)]
struct Student {
    name: String,
    scores: Vec<u32>,
}

#[derive(Debug, PartialEq)]
enum Grade {
    Excellent,
    Good,
    Pass,
    Fail,
}

impl Student {
    fn average(&self) -> f64 {
        if self.scores.is_empty() {
            return 0.0;
        }

        let sum: u32 = self.scores.iter().sum();
        sum as f64 / self.scores.len() as f64
    }

    fn get_grade(&self) -> Grade {
        let avg: f64 = self.average();
        if (avg >= 90.0) {
            Grade::Excellent
        } else if (avg >= 75.0) {
            Grade::Good
        } else if (avg >= 60.0) {
            Grade::Pass
        } else {
            Grade::Fail
        }
    }
}

fn main() {
    let students = vec![
        Student {
            name: "小铭".into(),
            scores: vec![78,88, 92],
        },
        Student {
            name: "小红".into(),
            scores: vec![90, 91, 60],
        },
        Student {
            name: "小李".into(),
            scores: vec![44, 60, 43],
        },
    ];
    for student in &students {
        println!(
            "{}：平均分 {:.1}，等级 {:?}",
            student.name,
            student.average(),
            student.get_grade()
        );
    }
    println!("共 {} 名学生", students.len());
}

