# -*- coding: utf-8 -*-

################################################################################
## Form generated from reading UI file 'MainWindow.ui'
##
## Created by: Qt User Interface Compiler version 6.7.1
##
## WARNING! All changes made in this file will be lost when recompiling UI file!
################################################################################

from PySide6.QtCore import (QCoreApplication, QDate, QDateTime, QLocale,
    QMetaObject, QObject, QPoint, QRect,
    QSize, QTime, QUrl, Qt)
from PySide6.QtGui import (QBrush, QColor, QConicalGradient, QCursor,
    QFont, QFontDatabase, QGradient, QIcon,
    QImage, QKeySequence, QLinearGradient, QPainter,
    QPalette, QPixmap, QRadialGradient, QTransform)
from PySide6.QtWidgets import (QAbstractScrollArea, QApplication, QCheckBox, QComboBox,
    QFormLayout, QGridLayout, QGroupBox, QHBoxLayout,
    QLabel, QLineEdit, QMainWindow, QPushButton,
    QRadioButton, QScrollArea, QSizePolicy, QSpacerItem,
    QSpinBox, QTabWidget, QVBoxLayout, QWidget)

class Ui_MotionFrame(object):
    def setupUi(self, MotionFrame):
        if not MotionFrame.objectName():
            MotionFrame.setObjectName(u"MotionFrame")
        MotionFrame.resize(895, 747)
        self.central = QWidget(MotionFrame)
        self.central.setObjectName(u"central")
        self.horizontalLayout = QHBoxLayout(self.central)
        self.horizontalLayout.setObjectName(u"horizontalLayout")
        self.scrollArea = QScrollArea(self.central)
        self.scrollArea.setObjectName(u"scrollArea")
        self.scrollArea.setMinimumSize(QSize(0, 0))
        self.scrollArea.setLineWidth(-1)
        self.scrollArea.setVerticalScrollBarPolicy(Qt.ScrollBarAlwaysOn)
        self.scrollArea.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOff)
        self.scrollArea.setSizeAdjustPolicy(QAbstractScrollArea.AdjustToContents)
        self.scrollArea.setWidgetResizable(True)
        self.scrollAreaWidgetContents_2 = QWidget()
        self.scrollAreaWidgetContents_2.setObjectName(u"scrollAreaWidgetContents_2")
        self.scrollAreaWidgetContents_2.setGeometry(QRect(0, 0, 499, 921))
        self.verticalLayout = QVBoxLayout(self.scrollAreaWidgetContents_2)
        self.verticalLayout.setObjectName(u"verticalLayout")
        self.form_config = QFormLayout()
        self.form_config.setObjectName(u"form_config")
        self.form_config.setFieldGrowthPolicy(QFormLayout.ExpandingFieldsGrow)
        self.form_config.setFormAlignment(Qt.AlignLeading|Qt.AlignLeft|Qt.AlignTop)
        self.label_language = QLabel(self.scrollAreaWidgetContents_2)
        self.label_language.setObjectName(u"label_language")
        sizePolicy = QSizePolicy(QSizePolicy.Policy.Minimum, QSizePolicy.Policy.Preferred)
        sizePolicy.setHorizontalStretch(0)
        sizePolicy.setVerticalStretch(0)
        sizePolicy.setHeightForWidth(self.label_language.sizePolicy().hasHeightForWidth())
        self.label_language.setSizePolicy(sizePolicy)
        self.label_language.setMinimumSize(QSize(215, 0))
        self.label_language.setTextFormat(Qt.PlainText)
        self.label_language.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.form_config.setWidget(0, QFormLayout.LabelRole, self.label_language)

        self.layout_language = QHBoxLayout()
        self.layout_language.setObjectName(u"layout_language")
        self.radio_button_language_english = QRadioButton(self.scrollAreaWidgetContents_2)
        self.radio_button_language_english.setObjectName(u"radio_button_language_english")
        self.radio_button_language_english.setText(u"English")
        self.radio_button_language_english.setChecked(True)

        self.layout_language.addWidget(self.radio_button_language_english)

        self.radio_button_language_japanese = QRadioButton(self.scrollAreaWidgetContents_2)
        self.radio_button_language_japanese.setObjectName(u"radio_button_language_japanese")
        self.radio_button_language_japanese.setText(u"\u65e5\u672c\u8a9e")

        self.layout_language.addWidget(self.radio_button_language_japanese)


        self.form_config.setLayout(0, QFormLayout.FieldRole, self.layout_language)


        self.verticalLayout.addLayout(self.form_config)

        self.groupbox_file_input = QGroupBox(self.scrollAreaWidgetContents_2)
        self.groupbox_file_input.setObjectName(u"groupbox_file_input")
        self.formLayout = QFormLayout(self.groupbox_file_input)
        self.formLayout.setObjectName(u"formLayout")
        self.formLayout.setFormAlignment(Qt.AlignLeading|Qt.AlignLeft|Qt.AlignTop)
        self.button_file_browse = QPushButton(self.groupbox_file_input)
        self.button_file_browse.setObjectName(u"button_file_browse")
        self.button_file_browse.setMaximumSize(QSize(221, 32))

        self.formLayout.setWidget(0, QFormLayout.FieldRole, self.button_file_browse)

        self.label_directory = QLabel(self.groupbox_file_input)
        self.label_directory.setObjectName(u"label_directory")
        sizePolicy.setHeightForWidth(self.label_directory.sizePolicy().hasHeightForWidth())
        self.label_directory.setSizePolicy(sizePolicy)
        self.label_directory.setMinimumSize(QSize(200, 0))
        self.label_directory.setTextFormat(Qt.PlainText)
        self.label_directory.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout.setWidget(1, QFormLayout.LabelRole, self.label_directory)

        self.text_directory = QLineEdit(self.groupbox_file_input)
        self.text_directory.setObjectName(u"text_directory")
        sizePolicy1 = QSizePolicy(QSizePolicy.Policy.Fixed, QSizePolicy.Policy.Fixed)
        sizePolicy1.setHorizontalStretch(0)
        sizePolicy1.setVerticalStretch(0)
        sizePolicy1.setHeightForWidth(self.text_directory.sizePolicy().hasHeightForWidth())
        self.text_directory.setSizePolicy(sizePolicy1)
        self.text_directory.setMinimumSize(QSize(200, 0))
        self.text_directory.setBaseSize(QSize(0, 0))

        self.formLayout.setWidget(1, QFormLayout.FieldRole, self.text_directory)

        self.label_file_prefix = QLabel(self.groupbox_file_input)
        self.label_file_prefix.setObjectName(u"label_file_prefix")
        sizePolicy.setHeightForWidth(self.label_file_prefix.sizePolicy().hasHeightForWidth())
        self.label_file_prefix.setSizePolicy(sizePolicy)
        self.label_file_prefix.setMinimumSize(QSize(200, 0))
        self.label_file_prefix.setTextFormat(Qt.PlainText)
        self.label_file_prefix.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout.setWidget(2, QFormLayout.LabelRole, self.label_file_prefix)

        self.text_file_prefix = QLineEdit(self.groupbox_file_input)
        self.text_file_prefix.setObjectName(u"text_file_prefix")
        sizePolicy1.setHeightForWidth(self.text_file_prefix.sizePolicy().hasHeightForWidth())
        self.text_file_prefix.setSizePolicy(sizePolicy1)
        self.text_file_prefix.setMinimumSize(QSize(200, 0))
        self.text_file_prefix.setText(u"flipbook_")

        self.formLayout.setWidget(2, QFormLayout.FieldRole, self.text_file_prefix)

        self.label_extension = QLabel(self.groupbox_file_input)
        self.label_extension.setObjectName(u"label_extension")
        sizePolicy.setHeightForWidth(self.label_extension.sizePolicy().hasHeightForWidth())
        self.label_extension.setSizePolicy(sizePolicy)
        self.label_extension.setMinimumSize(QSize(200, 0))
        self.label_extension.setTextFormat(Qt.PlainText)
        self.label_extension.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout.setWidget(3, QFormLayout.LabelRole, self.label_extension)

        self.text_extension = QLineEdit(self.groupbox_file_input)
        self.text_extension.setObjectName(u"text_extension")
        sizePolicy1.setHeightForWidth(self.text_extension.sizePolicy().hasHeightForWidth())
        self.text_extension.setSizePolicy(sizePolicy1)
        self.text_extension.setMinimumSize(QSize(200, 0))
        self.text_extension.setText(u"tga")

        self.formLayout.setWidget(3, QFormLayout.FieldRole, self.text_extension)

        self.label_sequence_digits = QLabel(self.groupbox_file_input)
        self.label_sequence_digits.setObjectName(u"label_sequence_digits")
        sizePolicy.setHeightForWidth(self.label_sequence_digits.sizePolicy().hasHeightForWidth())
        self.label_sequence_digits.setSizePolicy(sizePolicy)
        self.label_sequence_digits.setMinimumSize(QSize(200, 0))
        self.label_sequence_digits.setTextFormat(Qt.PlainText)
        self.label_sequence_digits.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout.setWidget(4, QFormLayout.LabelRole, self.label_sequence_digits)

        self.number_sequence_digits = QSpinBox(self.groupbox_file_input)
        self.number_sequence_digits.setObjectName(u"number_sequence_digits")
        self.number_sequence_digits.setMinimum(1)
        self.number_sequence_digits.setMaximum(10)
        self.number_sequence_digits.setValue(3)

        self.formLayout.setWidget(4, QFormLayout.FieldRole, self.number_sequence_digits)


        self.verticalLayout.addWidget(self.groupbox_file_input)

        self.groupbox_atlas = QGroupBox(self.scrollAreaWidgetContents_2)
        self.groupbox_atlas.setObjectName(u"groupbox_atlas")
        self.formLayout_2 = QFormLayout(self.groupbox_atlas)
        self.formLayout_2.setObjectName(u"formLayout_2")
        self.formLayout_2.setFormAlignment(Qt.AlignLeading|Qt.AlignLeft|Qt.AlignTop)
        self.label_atlas_resolution_width = QLabel(self.groupbox_atlas)
        self.label_atlas_resolution_width.setObjectName(u"label_atlas_resolution_width")
        sizePolicy.setHeightForWidth(self.label_atlas_resolution_width.sizePolicy().hasHeightForWidth())
        self.label_atlas_resolution_width.setSizePolicy(sizePolicy)
        self.label_atlas_resolution_width.setMinimumSize(QSize(200, 0))
        self.label_atlas_resolution_width.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout_2.setWidget(0, QFormLayout.LabelRole, self.label_atlas_resolution_width)

        self.combo_atlas_resolution_width = QComboBox(self.groupbox_atlas)
        self.combo_atlas_resolution_width.addItem(u"32")
        self.combo_atlas_resolution_width.addItem(u"64")
        self.combo_atlas_resolution_width.addItem(u"128")
        self.combo_atlas_resolution_width.addItem(u"256")
        self.combo_atlas_resolution_width.addItem(u"512")
        self.combo_atlas_resolution_width.addItem(u"1024")
        self.combo_atlas_resolution_width.addItem(u"2048")
        self.combo_atlas_resolution_width.addItem(u"4096")
        self.combo_atlas_resolution_width.addItem(u"8192")
        self.combo_atlas_resolution_width.addItem(u"16384")
        self.combo_atlas_resolution_width.setObjectName(u"combo_atlas_resolution_width")
        self.combo_atlas_resolution_width.setCurrentText(u"2048")

        self.formLayout_2.setWidget(0, QFormLayout.FieldRole, self.combo_atlas_resolution_width)

        self.label_atlas_width = QLabel(self.groupbox_atlas)
        self.label_atlas_width.setObjectName(u"label_atlas_width")
        sizePolicy.setHeightForWidth(self.label_atlas_width.sizePolicy().hasHeightForWidth())
        self.label_atlas_width.setSizePolicy(sizePolicy)
        self.label_atlas_width.setMinimumSize(QSize(200, 0))
        self.label_atlas_width.setTextFormat(Qt.PlainText)
        self.label_atlas_width.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout_2.setWidget(2, QFormLayout.LabelRole, self.label_atlas_width)

        self.label_atlas_height = QLabel(self.groupbox_atlas)
        self.label_atlas_height.setObjectName(u"label_atlas_height")
        sizePolicy.setHeightForWidth(self.label_atlas_height.sizePolicy().hasHeightForWidth())
        self.label_atlas_height.setSizePolicy(sizePolicy)
        self.label_atlas_height.setMinimumSize(QSize(200, 0))
        self.label_atlas_height.setTextFormat(Qt.PlainText)
        self.label_atlas_height.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout_2.setWidget(4, QFormLayout.LabelRole, self.label_atlas_height)

        self.label_stagger_pack = QLabel(self.groupbox_atlas)
        self.label_stagger_pack.setObjectName(u"label_stagger_pack")
        self.label_stagger_pack.setMinimumSize(QSize(200, 0))
        self.label_stagger_pack.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout_2.setWidget(5, QFormLayout.LabelRole, self.label_stagger_pack)

        self.checkbox_stagger_pack = QCheckBox(self.groupbox_atlas)
        self.checkbox_stagger_pack.setObjectName(u"checkbox_stagger_pack")
        self.checkbox_stagger_pack.setEnabled(True)
        self.checkbox_stagger_pack.setText(u"")

        self.formLayout_2.setWidget(5, QFormLayout.FieldRole, self.checkbox_stagger_pack)

        self.number_atlas_width = QComboBox(self.groupbox_atlas)
        self.number_atlas_width.addItem(u"2")
        self.number_atlas_width.addItem(u"4")
        self.number_atlas_width.addItem(u"8")
        self.number_atlas_width.addItem(u"16")
        self.number_atlas_width.addItem(u"32")
        self.number_atlas_width.addItem(u"64")
        self.number_atlas_width.addItem(u"128")
        self.number_atlas_width.addItem(u"256")
        self.number_atlas_width.setObjectName(u"number_atlas_width")

        self.formLayout_2.setWidget(2, QFormLayout.FieldRole, self.number_atlas_width)

        self.number_atlas_height = QComboBox(self.groupbox_atlas)
        self.number_atlas_height.addItem(u"2")
        self.number_atlas_height.addItem(u"4")
        self.number_atlas_height.addItem(u"8")
        self.number_atlas_height.addItem(u"16")
        self.number_atlas_height.addItem(u"32")
        self.number_atlas_height.addItem(u"64")
        self.number_atlas_height.addItem(u"128")
        self.number_atlas_height.addItem(u"256")
        self.number_atlas_height.setObjectName(u"number_atlas_height")

        self.formLayout_2.setWidget(4, QFormLayout.FieldRole, self.number_atlas_height)

        self.label_atlas_extrude = QLabel(self.groupbox_atlas)
        self.label_atlas_extrude.setObjectName(u"label_atlas_extrude")
        sizePolicy.setHeightForWidth(self.label_atlas_extrude.sizePolicy().hasHeightForWidth())
        self.label_atlas_extrude.setSizePolicy(sizePolicy)
        self.label_atlas_extrude.setMinimumSize(QSize(200, 0))
        self.label_atlas_extrude.setTextFormat(Qt.PlainText)
        self.label_atlas_extrude.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout_2.setWidget(1, QFormLayout.LabelRole, self.label_atlas_extrude)

        self.number_extrude = QSpinBox(self.groupbox_atlas)
        self.number_extrude.setObjectName(u"number_extrude")
        self.number_extrude.setMaximum(128)
        self.number_extrude.setValue(0)

        self.formLayout_2.setWidget(1, QFormLayout.FieldRole, self.number_extrude)


        self.verticalLayout.addWidget(self.groupbox_atlas)

        self.groupbox_animation = QGroupBox(self.scrollAreaWidgetContents_2)
        self.groupbox_animation.setObjectName(u"groupbox_animation")
        self.formLayout_3 = QFormLayout(self.groupbox_animation)
        self.formLayout_3.setObjectName(u"formLayout_3")
        self.formLayout_3.setFormAlignment(Qt.AlignLeading|Qt.AlignLeft|Qt.AlignTop)
        self.label_frame_skip = QLabel(self.groupbox_animation)
        self.label_frame_skip.setObjectName(u"label_frame_skip")
        sizePolicy.setHeightForWidth(self.label_frame_skip.sizePolicy().hasHeightForWidth())
        self.label_frame_skip.setSizePolicy(sizePolicy)
        self.label_frame_skip.setMinimumSize(QSize(200, 0))
        self.label_frame_skip.setTextFormat(Qt.PlainText)
        self.label_frame_skip.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout_3.setWidget(0, QFormLayout.LabelRole, self.label_frame_skip)

        self.number_frame_skip = QSpinBox(self.groupbox_animation)
        self.number_frame_skip.setObjectName(u"number_frame_skip")
        self.number_frame_skip.setMinimumSize(QSize(0, 0))
        self.number_frame_skip.setMinimum(0)
        self.number_frame_skip.setMaximum(128)
        self.number_frame_skip.setValue(0)

        self.formLayout_3.setWidget(0, QFormLayout.FieldRole, self.number_frame_skip)

        self.label_analyze_skipped_frames = QLabel(self.groupbox_animation)
        self.label_analyze_skipped_frames.setObjectName(u"label_analyze_skipped_frames")
        sizePolicy.setHeightForWidth(self.label_analyze_skipped_frames.sizePolicy().hasHeightForWidth())
        self.label_analyze_skipped_frames.setSizePolicy(sizePolicy)
        self.label_analyze_skipped_frames.setMinimumSize(QSize(200, 0))
        self.label_analyze_skipped_frames.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout_3.setWidget(1, QFormLayout.LabelRole, self.label_analyze_skipped_frames)

        self.checkbox_analyze_skipped_frames = QCheckBox(self.groupbox_animation)
        self.checkbox_analyze_skipped_frames.setObjectName(u"checkbox_analyze_skipped_frames")
        self.checkbox_analyze_skipped_frames.setText(u"")
        self.checkbox_analyze_skipped_frames.setChecked(True)

        self.formLayout_3.setWidget(1, QFormLayout.FieldRole, self.checkbox_analyze_skipped_frames)

        self.label_loop = QLabel(self.groupbox_animation)
        self.label_loop.setObjectName(u"label_loop")
        sizePolicy.setHeightForWidth(self.label_loop.sizePolicy().hasHeightForWidth())
        self.label_loop.setSizePolicy(sizePolicy)
        self.label_loop.setMinimumSize(QSize(200, 0))
        self.label_loop.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout_3.setWidget(2, QFormLayout.LabelRole, self.label_loop)

        self.checkbox_loop = QCheckBox(self.groupbox_animation)
        self.checkbox_loop.setObjectName(u"checkbox_loop")
        self.checkbox_loop.setText(u"")
        self.checkbox_loop.setCheckable(True)

        self.formLayout_3.setWidget(2, QFormLayout.FieldRole, self.checkbox_loop)


        self.verticalLayout.addWidget(self.groupbox_animation)

        self.groupbox_export = QGroupBox(self.scrollAreaWidgetContents_2)
        self.groupbox_export.setObjectName(u"groupbox_export")
        self.formLayout_4 = QFormLayout(self.groupbox_export)
        self.formLayout_4.setObjectName(u"formLayout_4")
        self.formLayout_4.setFormAlignment(Qt.AlignLeading|Qt.AlignLeft|Qt.AlignTop)
        self.label_downsample_motion_vector = QLabel(self.groupbox_export)
        self.label_downsample_motion_vector.setObjectName(u"label_downsample_motion_vector")
        sizePolicy.setHeightForWidth(self.label_downsample_motion_vector.sizePolicy().hasHeightForWidth())
        self.label_downsample_motion_vector.setSizePolicy(sizePolicy)
        self.label_downsample_motion_vector.setMinimumSize(QSize(200, 0))
        self.label_downsample_motion_vector.setTextFormat(Qt.PlainText)
        self.label_downsample_motion_vector.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout_4.setWidget(0, QFormLayout.LabelRole, self.label_downsample_motion_vector)

        self.checkbox_downsample_motion_vector = QCheckBox(self.groupbox_export)
        self.checkbox_downsample_motion_vector.setObjectName(u"checkbox_downsample_motion_vector")
        self.checkbox_downsample_motion_vector.setText(u"")
        self.checkbox_downsample_motion_vector.setChecked(True)

        self.formLayout_4.setWidget(0, QFormLayout.FieldRole, self.checkbox_downsample_motion_vector)

        self.label_motion_vector_encoding = QLabel(self.groupbox_export)
        self.label_motion_vector_encoding.setObjectName(u"label_motion_vector_encoding")
        sizePolicy.setHeightForWidth(self.label_motion_vector_encoding.sizePolicy().hasHeightForWidth())
        self.label_motion_vector_encoding.setSizePolicy(sizePolicy)
        self.label_motion_vector_encoding.setMinimumSize(QSize(200, 0))
        self.label_motion_vector_encoding.setTextFormat(Qt.PlainText)
        self.label_motion_vector_encoding.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout_4.setWidget(1, QFormLayout.LabelRole, self.label_motion_vector_encoding)

        self.combo_motion_vector_encoding = QComboBox(self.groupbox_export)
        self.combo_motion_vector_encoding.addItem(u"R8G8 Remapped to 0-1")
        self.combo_motion_vector_encoding.addItem(u"SideFX Labs R8G8 Encoding")
        self.combo_motion_vector_encoding.setObjectName(u"combo_motion_vector_encoding")

        self.formLayout_4.setWidget(1, QFormLayout.FieldRole, self.combo_motion_vector_encoding)

        self.label_resize_algorithm = QLabel(self.groupbox_export)
        self.label_resize_algorithm.setObjectName(u"label_resize_algorithm")
        self.label_resize_algorithm.setMinimumSize(QSize(200, 0))
        self.label_resize_algorithm.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.formLayout_4.setWidget(2, QFormLayout.LabelRole, self.label_resize_algorithm)

        self.combo_resize_algorithm = QComboBox(self.groupbox_export)
        self.combo_resize_algorithm.addItem(u"Cubic")
        self.combo_resize_algorithm.addItem(u"Bilinear")
        self.combo_resize_algorithm.setObjectName(u"combo_resize_algorithm")

        self.formLayout_4.setWidget(2, QFormLayout.FieldRole, self.combo_resize_algorithm)


        self.verticalLayout.addWidget(self.groupbox_export)

        self.button_generate = QPushButton(self.scrollAreaWidgetContents_2)
        self.button_generate.setObjectName(u"button_generate")

        self.verticalLayout.addWidget(self.button_generate)

        self.verticalSpacer = QSpacerItem(20, 13, QSizePolicy.Policy.Minimum, QSizePolicy.Policy.Minimum)

        self.verticalLayout.addItem(self.verticalSpacer)

        self.groupBox = QGroupBox(self.scrollAreaWidgetContents_2)
        self.groupBox.setObjectName(u"groupBox")
        self.form_input_frames = QFormLayout(self.groupBox)
        self.form_input_frames.setObjectName(u"form_input_frames")
        self.form_input_frames.setFormAlignment(Qt.AlignLeading|Qt.AlignLeft|Qt.AlignTop)
        self.label_input_number_of_frames = QLabel(self.groupBox)
        self.label_input_number_of_frames.setObjectName(u"label_input_number_of_frames")
        sizePolicy.setHeightForWidth(self.label_input_number_of_frames.sizePolicy().hasHeightForWidth())
        self.label_input_number_of_frames.setSizePolicy(sizePolicy)
        self.label_input_number_of_frames.setMinimumSize(QSize(200, 0))
        self.label_input_number_of_frames.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.form_input_frames.setWidget(0, QFormLayout.LabelRole, self.label_input_number_of_frames)

        self.label_input_number_of_frames_value = QLabel(self.groupBox)
        self.label_input_number_of_frames_value.setObjectName(u"label_input_number_of_frames_value")
        self.label_input_number_of_frames_value.setText(u"0")

        self.form_input_frames.setWidget(0, QFormLayout.FieldRole, self.label_input_number_of_frames_value)

        self.label_optimal_input_number_of_frames = QLabel(self.groupBox)
        self.label_optimal_input_number_of_frames.setObjectName(u"label_optimal_input_number_of_frames")
        sizePolicy.setHeightForWidth(self.label_optimal_input_number_of_frames.sizePolicy().hasHeightForWidth())
        self.label_optimal_input_number_of_frames.setSizePolicy(sizePolicy)
        self.label_optimal_input_number_of_frames.setMinimumSize(QSize(200, 0))
        self.label_optimal_input_number_of_frames.setAlignment(Qt.AlignRight|Qt.AlignTrailing|Qt.AlignVCenter)

        self.form_input_frames.setWidget(1, QFormLayout.LabelRole, self.label_optimal_input_number_of_frames)

        self.label_optimal_input_number_of_frames_value = QLabel(self.groupBox)
        self.label_optimal_input_number_of_frames_value.setObjectName(u"label_optimal_input_number_of_frames_value")
        self.label_optimal_input_number_of_frames_value.setText(u"0")

        self.form_input_frames.setWidget(1, QFormLayout.FieldRole, self.label_optimal_input_number_of_frames_value)


        self.verticalLayout.addWidget(self.groupBox)

        self.button_update_frames = QPushButton(self.scrollAreaWidgetContents_2)
        self.button_update_frames.setObjectName(u"button_update_frames")

        self.verticalLayout.addWidget(self.button_update_frames)

        self.label_motion_vector = QLabel(self.scrollAreaWidgetContents_2)
        self.label_motion_vector.setObjectName(u"label_motion_vector")

        self.verticalLayout.addWidget(self.label_motion_vector)

        self.left_spacer = QSpacerItem(378, 13, QSizePolicy.Policy.Minimum, QSizePolicy.Policy.Expanding)

        self.verticalLayout.addItem(self.left_spacer)

        self.scrollArea.setWidget(self.scrollAreaWidgetContents_2)

        self.horizontalLayout.addWidget(self.scrollArea)

        self.layout_right = QVBoxLayout()
        self.layout_right.setObjectName(u"layout_right")
        self.tabs_result = QTabWidget(self.central)
        self.tabs_result.setObjectName(u"tabs_result")
        self.tabs_result.setTabPosition(QTabWidget.North)
        self.tabs_result.setTabShape(QTabWidget.Rounded)
        self.tabs_result.setUsesScrollButtons(False)
        self.tab_color = QWidget()
        self.tab_color.setObjectName(u"tab_color")
        self.gridLayout = QGridLayout(self.tab_color)
        self.gridLayout.setObjectName(u"gridLayout")
        self.scroll_color = QScrollArea(self.tab_color)
        self.scroll_color.setObjectName(u"scroll_color")
        self.scroll_color.setVerticalScrollBarPolicy(Qt.ScrollBarAlwaysOn)
        self.scroll_color.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOn)
        self.scroll_color.setWidgetResizable(True)
        self.scroll_content_color = QWidget()
        self.scroll_content_color.setObjectName(u"scroll_content_color")
        self.scroll_content_color.setGeometry(QRect(0, 0, 284, 257))
        self.gridLayout_2 = QGridLayout(self.scroll_content_color)
        self.gridLayout_2.setObjectName(u"gridLayout_2")
        self.label_color_atlas_image = QLabel(self.scroll_content_color)
        self.label_color_atlas_image.setObjectName(u"label_color_atlas_image")

        self.gridLayout_2.addWidget(self.label_color_atlas_image, 0, 0, 1, 1)

        self.scroll_color.setWidget(self.scroll_content_color)

        self.gridLayout.addWidget(self.scroll_color, 0, 0, 1, 1)

        self.tabs_result.addTab(self.tab_color, "")
        self.tab_motion_vector = QWidget()
        self.tab_motion_vector.setObjectName(u"tab_motion_vector")
        self.gridLayout_3 = QGridLayout(self.tab_motion_vector)
        self.gridLayout_3.setObjectName(u"gridLayout_3")
        self.scroll_motion_vector = QScrollArea(self.tab_motion_vector)
        self.scroll_motion_vector.setObjectName(u"scroll_motion_vector")
        self.scroll_motion_vector.setVerticalScrollBarPolicy(Qt.ScrollBarAlwaysOn)
        self.scroll_motion_vector.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOn)
        self.scroll_motion_vector.setWidgetResizable(True)
        self.scroll_content_motion_vector = QWidget()
        self.scroll_content_motion_vector.setObjectName(u"scroll_content_motion_vector")
        self.scroll_content_motion_vector.setGeometry(QRect(0, 0, 98, 40))
        self.gridLayout_4 = QGridLayout(self.scroll_content_motion_vector)
        self.gridLayout_4.setObjectName(u"gridLayout_4")
        self.label_motion_vector_image = QLabel(self.scroll_content_motion_vector)
        self.label_motion_vector_image.setObjectName(u"label_motion_vector_image")

        self.gridLayout_4.addWidget(self.label_motion_vector_image, 0, 0, 1, 1)

        self.scroll_motion_vector.setWidget(self.scroll_content_motion_vector)

        self.gridLayout_3.addWidget(self.scroll_motion_vector, 0, 0, 1, 1)

        self.tabs_result.addTab(self.tab_motion_vector, "")
        self.tab_visualization = QWidget()
        self.tab_visualization.setObjectName(u"tab_visualization")
        self.gridLayout_6 = QGridLayout(self.tab_visualization)
        self.gridLayout_6.setObjectName(u"gridLayout_6")
        self.scroll_visualization = QScrollArea(self.tab_visualization)
        self.scroll_visualization.setObjectName(u"scroll_visualization")
        self.scroll_visualization.setVerticalScrollBarPolicy(Qt.ScrollBarAlwaysOn)
        self.scroll_visualization.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOn)
        self.scroll_visualization.setWidgetResizable(True)
        self.scroll_content_motion_vector_2 = QWidget()
        self.scroll_content_motion_vector_2.setObjectName(u"scroll_content_motion_vector_2")
        self.scroll_content_motion_vector_2.setGeometry(QRect(0, 0, 98, 40))
        self.gridLayout_5 = QGridLayout(self.scroll_content_motion_vector_2)
        self.gridLayout_5.setObjectName(u"gridLayout_5")
        self.label_visualization_image = QLabel(self.scroll_content_motion_vector_2)
        self.label_visualization_image.setObjectName(u"label_visualization_image")

        self.gridLayout_5.addWidget(self.label_visualization_image, 0, 0, 1, 1)

        self.scroll_visualization.setWidget(self.scroll_content_motion_vector_2)

        self.gridLayout_6.addWidget(self.scroll_visualization, 0, 0, 1, 1)

        self.tabs_result.addTab(self.tab_visualization, "")

        self.layout_right.addWidget(self.tabs_result)

        self.form_result = QFormLayout()
        self.form_result.setObjectName(u"form_result")
        self.label_motion_strength = QLabel(self.central)
        self.label_motion_strength.setObjectName(u"label_motion_strength")
        self.label_motion_strength.setTextFormat(Qt.PlainText)

        self.form_result.setWidget(3, QFormLayout.LabelRole, self.label_motion_strength)

        self.label_discarded_trailing_frames = QLabel(self.central)
        self.label_discarded_trailing_frames.setObjectName(u"label_discarded_trailing_frames")
        self.label_discarded_trailing_frames.setTextFormat(Qt.PlainText)

        self.form_result.setWidget(1, QFormLayout.LabelRole, self.label_discarded_trailing_frames)

        self.label_discarded_trailing_frames_value = QLabel(self.central)
        self.label_discarded_trailing_frames_value.setObjectName(u"label_discarded_trailing_frames_value")
        self.label_discarded_trailing_frames_value.setText(u"0")
        self.label_discarded_trailing_frames_value.setTextFormat(Qt.PlainText)

        self.form_result.setWidget(1, QFormLayout.FieldRole, self.label_discarded_trailing_frames_value)

        self.layout_motion_strength = QHBoxLayout()
        self.layout_motion_strength.setObjectName(u"layout_motion_strength")
        self.text_motion_strength = QLineEdit(self.central)
        self.text_motion_strength.setObjectName(u"text_motion_strength")
        self.text_motion_strength.setReadOnly(True)

        self.layout_motion_strength.addWidget(self.text_motion_strength)

        self.button_copy_motion_strength = QPushButton(self.central)
        self.button_copy_motion_strength.setObjectName(u"button_copy_motion_strength")

        self.layout_motion_strength.addWidget(self.button_copy_motion_strength)


        self.form_result.setLayout(3, QFormLayout.FieldRole, self.layout_motion_strength)

        self.label_total_frames = QLabel(self.central)
        self.label_total_frames.setObjectName(u"label_total_frames")

        self.form_result.setWidget(0, QFormLayout.LabelRole, self.label_total_frames)

        self.label_total_frames_value = QLabel(self.central)
        self.label_total_frames_value.setObjectName(u"label_total_frames_value")
        self.label_total_frames_value.setText(u"0")

        self.form_result.setWidget(0, QFormLayout.FieldRole, self.label_total_frames_value)


        self.layout_right.addLayout(self.form_result)

        self.button_save = QPushButton(self.central)
        self.button_save.setObjectName(u"button_save")

        self.layout_right.addWidget(self.button_save)


        self.horizontalLayout.addLayout(self.layout_right)

        self.horizontalLayout.setStretch(1, 1)
        MotionFrame.setCentralWidget(self.central)
        QWidget.setTabOrder(self.radio_button_language_english, self.radio_button_language_japanese)
        QWidget.setTabOrder(self.radio_button_language_japanese, self.button_file_browse)
        QWidget.setTabOrder(self.button_file_browse, self.text_directory)
        QWidget.setTabOrder(self.text_directory, self.text_file_prefix)
        QWidget.setTabOrder(self.text_file_prefix, self.text_extension)
        QWidget.setTabOrder(self.text_extension, self.number_sequence_digits)
        QWidget.setTabOrder(self.number_sequence_digits, self.combo_atlas_resolution_width)
        QWidget.setTabOrder(self.combo_atlas_resolution_width, self.number_extrude)
        QWidget.setTabOrder(self.number_extrude, self.number_atlas_width)
        QWidget.setTabOrder(self.number_atlas_width, self.number_atlas_height)
        QWidget.setTabOrder(self.number_atlas_height, self.checkbox_stagger_pack)
        QWidget.setTabOrder(self.checkbox_stagger_pack, self.number_frame_skip)
        QWidget.setTabOrder(self.number_frame_skip, self.checkbox_analyze_skipped_frames)
        QWidget.setTabOrder(self.checkbox_analyze_skipped_frames, self.checkbox_loop)
        QWidget.setTabOrder(self.checkbox_loop, self.checkbox_downsample_motion_vector)
        QWidget.setTabOrder(self.checkbox_downsample_motion_vector, self.combo_motion_vector_encoding)
        QWidget.setTabOrder(self.combo_motion_vector_encoding, self.combo_resize_algorithm)
        QWidget.setTabOrder(self.combo_resize_algorithm, self.scroll_visualization)
        QWidget.setTabOrder(self.scroll_visualization, self.text_motion_strength)
        QWidget.setTabOrder(self.text_motion_strength, self.button_copy_motion_strength)
        QWidget.setTabOrder(self.button_copy_motion_strength, self.button_save)
        QWidget.setTabOrder(self.button_save, self.scrollArea)
        QWidget.setTabOrder(self.scrollArea, self.tabs_result)
        QWidget.setTabOrder(self.tabs_result, self.scroll_motion_vector)
        QWidget.setTabOrder(self.scroll_motion_vector, self.button_generate)
        QWidget.setTabOrder(self.button_generate, self.button_update_frames)
        QWidget.setTabOrder(self.button_update_frames, self.scroll_color)

        self.retranslateUi(MotionFrame)

        self.combo_atlas_resolution_width.setCurrentIndex(6)
        self.number_atlas_width.setCurrentIndex(2)
        self.number_atlas_height.setCurrentIndex(2)
        self.tabs_result.setCurrentIndex(0)


        QMetaObject.connectSlotsByName(MotionFrame)
    # setupUi

    def retranslateUi(self, MotionFrame):
        MotionFrame.setWindowTitle(QCoreApplication.translate("MotionFrame", u"MainWindow", None))
        self.label_language.setText(QCoreApplication.translate("MotionFrame", u"Language:", None))
        self.groupbox_file_input.setTitle(QCoreApplication.translate("MotionFrame", u"File Input", None))
        self.button_file_browse.setText(QCoreApplication.translate("MotionFrame", u"Browse\u2026", None))
        self.label_directory.setText(QCoreApplication.translate("MotionFrame", u"Directory:", None))
        self.label_file_prefix.setText(QCoreApplication.translate("MotionFrame", u"File Prefix:", None))
        self.label_extension.setText(QCoreApplication.translate("MotionFrame", u"Extension:", None))
        self.label_sequence_digits.setText(QCoreApplication.translate("MotionFrame", u"Sequence Digits:", None))
        self.groupbox_atlas.setTitle(QCoreApplication.translate("MotionFrame", u"Atlas", None))
        self.label_atlas_resolution_width.setText(QCoreApplication.translate("MotionFrame", u"Pixel Width:", None))

        self.label_atlas_width.setText(QCoreApplication.translate("MotionFrame", u"Columns (X):", None))
        self.label_atlas_height.setText(QCoreApplication.translate("MotionFrame", u"Rows (Y):", None))
        self.label_stagger_pack.setText(QCoreApplication.translate("MotionFrame", u"Stagger Pack:", None))


        self.label_atlas_extrude.setText(QCoreApplication.translate("MotionFrame", u"Extrude:", None))
        self.groupbox_animation.setTitle(QCoreApplication.translate("MotionFrame", u"Animation", None))
        self.label_frame_skip.setText(QCoreApplication.translate("MotionFrame", u"Frame Skip:", None))
        self.label_analyze_skipped_frames.setText(QCoreApplication.translate("MotionFrame", u"Analyze Skipped Frames:", None))
        self.label_loop.setText(QCoreApplication.translate("MotionFrame", u"Loop:", None))
        self.groupbox_export.setTitle(QCoreApplication.translate("MotionFrame", u"Export", None))
        self.label_downsample_motion_vector.setText(QCoreApplication.translate("MotionFrame", u"Downsample Motion Vector:", None))
        self.label_motion_vector_encoding.setText(QCoreApplication.translate("MotionFrame", u"Motion Vector Encoding:", None))

        self.label_resize_algorithm.setText(QCoreApplication.translate("MotionFrame", u"Resize Algorithm:", None))

        self.button_generate.setText(QCoreApplication.translate("MotionFrame", u"Generate", None))
        self.groupBox.setTitle(QCoreApplication.translate("MotionFrame", u"Input Frames", None))
        self.label_input_number_of_frames.setText(QCoreApplication.translate("MotionFrame", u"Current Frames:", None))
        self.label_optimal_input_number_of_frames.setText(QCoreApplication.translate("MotionFrame", u"Optimal Frames:", None))
        self.button_update_frames.setText(QCoreApplication.translate("MotionFrame", u"Recount Input Frames", None))
        self.label_motion_vector.setText("")
        self.label_color_atlas_image.setText("")
        self.tabs_result.setTabText(self.tabs_result.indexOf(self.tab_color), QCoreApplication.translate("MotionFrame", u"Color", None))
        self.label_motion_vector_image.setText("")
        self.tabs_result.setTabText(self.tabs_result.indexOf(self.tab_motion_vector), QCoreApplication.translate("MotionFrame", u"Motion Vector", None))
        self.label_visualization_image.setText("")
        self.tabs_result.setTabText(self.tabs_result.indexOf(self.tab_visualization), QCoreApplication.translate("MotionFrame", u"Visualization", None))
        self.label_motion_strength.setText(QCoreApplication.translate("MotionFrame", u"Motion Strength:", None))
        self.label_discarded_trailing_frames.setText(QCoreApplication.translate("MotionFrame", u"Discarded Trailing Frames:", None))
        self.button_copy_motion_strength.setText(QCoreApplication.translate("MotionFrame", u"Copy", None))
        self.label_total_frames.setText(QCoreApplication.translate("MotionFrame", u"Total Frames:", None))
        self.button_save.setText(QCoreApplication.translate("MotionFrame", u"Save", None))
    # retranslateUi

