import json
import bz2
import logging
import sqlite3
from typing import cast, List, Any, Iterable, BinaryIO, Optional

import dataset
import dataset.budgetary
import dataset.experimental_data
import dataset.estimation_result
import dataset.consistency_result
import dataset.experiment_stats

import branding
from gui.progress import Worker, Cancelled
from util.codec import Codec, FileIn, FileOut, strC, intC, listC
from util.codec_progress import CodecProgress, listCP, oneCP, enum_by_typenameCP

log = logging.getLogger(__name__)

SQL_FORMAT_VERSION = 16

class PersistenceError(Exception):
    pass

class Workspace:
    def __init__(self):
        self.db = sqlite3.connect(':memory:')
        self.file_name : Optional[str] = None

        with self.db.cursor() as cur:
            cur.execute('PRAGMA foreign_keys = 1')
            cur.execute('''
                CREATE TABLE meta (
                    name TEXT NOT NULL PRIMARY KEY,
                    value TEXT NOT NULL,
                );

                INSERT INTO meta VALUES
                    ('sql_format_version', %s),
                    ('prest_version', %s);

                CREATE TABLE dataset (
                    id INTEGER NOT NULL PRIMARY KEY,
                    name TEXT NOT NULL,
                );

                CREATE TABLE alternative (
                    dataset_id INTEGER NOT NULL,
                    alt_number INTEGER NOT NULL,

                    PRIMARY KEY (dataset_id, alt_number),
                    FOREIGN KEY (dataset_id) REFERENCES dataset(id)
                );

                CREATE TABLE subject (
                    id INTEGER NOT NULL PRIMARY KEY,
                    name TEXT NOT NULL,
                    dataset_id INTEGER NOT NULL,

                    FOREIGN KEY (dataset_id) REFERENCES dataset(id)
                );

                CREATE TABLE observation (
                    id INTEGER NOT NULL PRIMARY KEY,
                    subject_id INTEGER NOT NULL,
                    menu TEXT NOT NULL,
                    default TEXT NOT NULL,
                    choice TEXT NOT NULL,
                    
                    FOREIGN KEY (subject_id) REFERENCES subject(id)
                );
            ''', SQL_FORMAT_VERSION, branding.VERSION)

    def save_to_file(self, worker : Worker, fname: str) -> None:
        if fname == self.file_name:
            return  # nothing to do, the changes should already be in

        db_old = self.db
        db_new = sqlite3.connect(fname)
        db_new.execute('PRAGMA foreign_keys = 1')

        # TODO: progress=...
        db_old.backup(db_new)

        self.db = db_new
        self.file_name = fname
        db_old.close()

    def load_from_file(self, worker : Worker, fname: str) -> None:
        db_new = sqlite3.connect(fname)
        db_new.execute('PRAGMA foreign_keys = 1')

        # TODO: check version
        # TODO: run migrations

        self.db.close()
        self.db = db_new
        self.file_name = fname
