#!/usr/bin/env python3
import os
import cv2
import numpy as np
import imageio.v3 as imageio
import re
from PySide6.QtWidgets import (QApplication, QWidget, QLabel, QPushButton, QRadioButton, QVBoxLayout, QHBoxLayout, QGridLayout, QGroupBox, QFileDialog, QMessageBox, QTabWidget, QScrollArea, QLineEdit, QSpinBox, QMainWindow)
from PySide6.QtGui import QPixmap, QImage
from PySide6.QtCore import Qt, QSize, QTranslator, QLocale, QLibraryInfo
from PIL import Image
from MainWindow import Ui_MotionFrame
from pathlib import Path
import lib

class MotionFrameApp(QMainWindow, Ui_MotionFrame):
    def __init__(self):
        super(MotionFrameApp, self).__init__()
        self.setupUi(self)

        #super().__init__()
        self.setWindowTitle('MotionFrame')
        self.language = 'en'
        self.directory = None

        self.translator = QTranslator()
        self._load_translator()

        self.button_directory_browse.clicked.connect(self.select_directory)
        self.button_generate.clicked.connect(self.start_processing)
        self.button_copy_motion_strength.clicked.connect(self.copy_motion_strength)

        self.radio_button_language_english.toggled.connect(lambda: self.change_language('en'))
        self.radio_button_language_japanese.toggled.connect(lambda: self.change_language('ja'))

    def _get_path_qm(self, lang):
        return (Path(__file__).parent / f"./translation/motionframe_{lang}.qm").absolute().as_posix()

    def _load_translator(self):
        if self.language == 'ja':
            self.translator.load(self._get_path_qm(self.language))
            app.installTranslator(self.translator)

    def change_language(self, language):
        if language != self.language:
            self.language = language
            if language == 'en':
                app.removeTranslator(self.translator)
            else:
                self.translator.load(self._get_path_qm(self.language))
                app.installTranslator(self.translator)
            self.retranslateUi(self)

    def select_directory(self):
        directory = QFileDialog.getExistingDirectory(self, self.tr('Select Directory'))

        if directory:
            self.directory = directory
            self.text_directory.setText(directory)
            prefix, ext, zeros = self._detect_image_sequence_pattern()
            self.text_file_prefix.setText(prefix)
            self.text_extension.setText(ext)
            self.number_sequence_digits.setValue(zeros)

    def _detect_image_sequence_pattern(self):
        directory_path = self.directory

        files = os.listdir(directory_path)
        image_files = [f for f in files if re.search(r'\.(jpg|jpeg|png|bmp|tiff|tga)$', f, re.IGNORECASE)]

        if not image_files:
            return None, None, None

        pattern = re.compile(r'(.*?)(\d+)\.(\w+)$')

        prefixes = []
        numbers = []
        extensions = []

        for image_file in image_files:
            match = pattern.match(image_file)
            if match:
                prefix, number, extension = match.groups()
                prefixes.append(prefix)
                numbers.append(number)
                extensions.append(extension)

        if not prefixes:
            return None, None, None

        common_prefix = prefixes[0]
        extension = extensions[0]
        number_of_zeros = len(numbers[0])

        return common_prefix, extension, number_of_zeros

    def copy_motion_strength(self):
        clipboard = QApplication.clipboard()
        clipboard.setText(self.text_motion_strength.text())

    def _check_atlas_fit(self, atlas_width, atlas_height, frame_skip, total_frame_count):
        expected_frames = atlas_width * atlas_height
        actual_frames = lib.calculate_required_frames(total_frame_count, frame_skip)

        if actual_frames <= expected_frames:
            return True, self.tr('Frames can fit into the atlas.')
        else:
            excess_frames = actual_frames - expected_frames
            if excess_frames == 1:
                message = self.tr('1 frame won\'t fit into the atlas.')
            else:
                message = self.tr('%n frame(s) won\'t fit into the atlas.', '', excess_frames)

            min_skip = 0
            while lib.calculate_required_frames(total_frame_count, min_skip) > expected_frames:
                min_skip += 1

            message += '\n'
            message += self.tr('Try reducing the frame skip count or increasing the atlas size.')
            message += '\n'
            message += self.tr('Minimum frame skip required to fit the frames is %n.', '', min_skip)

            return False, message

    def start_processing(self):
        if not self.directory:
            QMessageBox.critical(self, self.tr('Error'), self.tr('No frames loaded. Check the file pattern and paths.'))
            return

        file_prefix = self.text_file_prefix.text()
        extension = self.text_extension.text()
        num_digits = self.number_sequence_digits.value()
        atlas_width = self.number_atlas_width.value()
        atlas_height = self.number_atlas_height.value()
        frame_skip = self.number_frame_skip.value()
        motion_scale = self.number_scale.value()

        pattern = os.path.join(self.directory, f"{file_prefix}%0{int(num_digits)}d.{extension}")

        frames = lib.load_frames(pattern)

        if not frames:
            QMessageBox.critical(self, self.tr('Error'), self.tr('No frames loaded. Check the file pattern and paths.'))
            return

        can_fit, error_message = self._check_atlas_fit(atlas_width, atlas_height, frame_skip, len(frames))
        if not can_fit:
            QMessageBox.critical(self, self.tr('Error'), error_message)
            return

        self.result = lib.encode_atlas(frames, atlas_width, atlas_height, frame_skip, motion_scale)

        self.display_image(self.result.color_atlas, self.label_color_atlas_image)
        self.display_image(self.result.motion_atlas, self.label_motion_vector_image)
        self.display_image(self.result.flow_directions, self.label_visualization_image)

        discarded_frames = len(frames) - ((lib.calculate_required_frames(len(frames), frame_skip) - 1) * (1 + frame_skip) - 1)
        self.text_motion_strength.setText(f"{self.result.strength:.8f}")
        self.label_discarded_trailing_frames_value.setText(str(discarded_frames))

    def display_image(self, image, label):
        channels = lib.channel_count(image)

        if channels == 3 or channels == 4:
            image_copy = np.zeros_like(image)
            # RGB to BGR conversion
            if channels == 3:
                image_copy[..., [0, 1, 2]] = image[..., [2, 1, 0]]
            elif channels == 4:
                image_copy[..., [0, 1, 2, 3]] = image[..., [2, 1, 0, 3]]
            image = image_copy

        height, width = image.shape[0], image.shape[1]

        bytes_per_line = channels * width
        image_format = QImage.Format_RGBA8888
        if channels == 3:
            image_format = QImage.Format_RGB888

        q_img = QImage(image.data, width, height, bytes_per_line, image_format)
        pixmap = QPixmap(q_img)
        label.setPixmap(pixmap)
        label.setAlignment(Qt.AlignCenter)

    def save_results(self):
        save_path = QFileDialog.getSaveFileName(self, self.tr('Save'))[0]
        if not save_path:
            return

        color_atlas_path = save_path + "_color_atlas.png"
        motion_atlas_path = save_path + "_motion_atlas.png"

        Image.fromarray(self.result.color_atlas).save(color_atlas_path)
        Image.fromarray(self.result.motion_atlas).save(motion_atlas_path)

        QMessageBox.information(self, self.tr('Save'), self.tr('The results have been saved successfully.'))

if __name__ == '__main__':
    app = QApplication([])
    window = MotionFrameApp()
    window.show()
    app.exec()
