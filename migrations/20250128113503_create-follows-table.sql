CREATE TABLE IF NOT EXISTS `follows` (
    `source` INTEGER NOT NULL,
    `target` INTEGER NOT NULL,
    FOREIGN KEY (`source`) REFERENCES `users`(`id`),
    FOREIGN KEY (`target`) REFERENCES `users`(`id`)
)
