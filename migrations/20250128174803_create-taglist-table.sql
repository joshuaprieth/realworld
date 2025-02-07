CREATE TABLE IF NOT EXISTS `taglist` (
    `article` INTEGER NOT NULL,
    `tag` INTEGER NOT NULL,
    FOREIGN KEY (`article`) REFERENCES `articles`(`id`),
    FOREIGN KEY (`tag`) REFERENCES `tags`(`id`)
)
