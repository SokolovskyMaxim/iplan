import gi
from gi.repository import Gtk, Adw, GLib

from iplan.db.operations.project import delete_project, read_projects

@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/project/project_delete_dialog.ui")
class ProjectDeleteDialog(Adw.MessageDialog):
    __gtype_name__ = "ProjectDeleteDialog"
    app = None

    def __init__(self, app):
        super().__init__()
        self.app = app
        self.set_heading(
            f'Delete "{self.app.project.name}" Project?'
        )

    @Gtk.Template.Callback()
    def on_responsed(self, dialog, response):
        if response == "delete":
            delete_project(self.app.project._id)
            self.activate_action("app.update_project")

            projects = read_projects()
            if not projects:
               self.app.project = read_projects(archive=True)[0]
            self.app.project = list(projects)[0]

            self.app.activate_action("open_project", GLib.Variant("i", -1))
            self.get_transient_for().close()

