# Git Auto Commit

WARN: This project is work-in-progress but also halted. I concluded that the
overall design is not the best. I don’t want to have an automated tool
that auto commits changes from a manually-curated instance. It would be better
to integrate a commit directly into the things that perform changes.

This is a small utility that makes Git commits if specified files have changed.

I have various cron jobs that do background changes in Git repositories
(package lock files, marks, etc.). This plugin helps automatically commit them.

This project is also an opportunity to use Rust.
