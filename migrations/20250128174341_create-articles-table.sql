CREATE TABLE IF NOT EXISTS `articles` (
    `id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL UNIQUE,
    `slug` TEXT NOT NULL UNIQUE,
    `title` TEXT NOT NULL,
    `description` TEXT NOT NULL,
    `body` TEXT NOT NULL,
    `createdAt` TEXT NOT NULL,
    `updatedAt` TEXT NOT NULL,
    `author` INTEGER NOT NULL,
    FOREIGN KEY (`author`) REFERENCES `users`(`id`)
)
