'''
Created on Oct 22, 2023

Intended intepreter: Jython 2.7 standalone jar

@author: xis31
'''
from burp import IBurpExtender
from burp import IContextMenuFactory, IContextMenuInvocation
from java.awt import Toolkit
from java.awt.datatransfer import StringSelection
from javax.swing import JMenuItem
import os 



class BurpExtender(IBurpExtender, IContextMenuFactory):
     
     tmp_folder_path = "C:/Users/xis31/tmp/"
     tmp_cache_path = "C:/Users/xis31/tmp/" + "req_cache.dat"
     cache_file = None
     bytes_ = None

     def registerExtenderCallbacks(self,callbacks):
        callbacks.setExtensionName("Intr-F1")
        file_stat = self.init_file()
        if file_stat == False:
            print("[-] Init failure...unable to read" + self.tmp_cache_path)
            exit()

        self.verify_tmp_folder()
        print("[+] Verifed tmp folder!")
        print("[+] Init!")
        callbacks.registerContextMenuFactory(self)
     
     def createMenuItems(self, ictx):
        ctx = ictx.getInvocationContext()
        if ctx == IContextMenuInvocation.CONTEXT_MESSAGE_EDITOR_REQUEST \
        or ctx == IContextMenuInvocation.CONTEXT_MESSAGE_VIEWER_REQUEST:
            self.bytes_ = self.get_req_bytes(ictx)
            label = "Send Request to Intr-F1"
            menuitem = JMenuItem(label, actionPerformed=self.sendtointr)
            print("[+] Menu item created")
            return [menuitem]
        
        
     def verify_tmp_folder(self):
         if os.path.exists(self.tmp_folder_path) != True:
             os.makedir(self.tmp_folder_path)
             
         if os.path.exists(self.tmp_cache_path) != True:
            f = open(self.tmp_cache_path)
            if f == False:
                return True
            
     def init_file(self):
         try: 
            cache_file = open(self.tmp_cache_path, "wb+")
         except IOError:
            self.cache_file = False
            return False
         self.cache_file = cache_file
         return True

     def get_req_bytes(self, invoc):
        if invoc != None:
            return invoc.getSelectedMessages()[0].getRequest()
        else:
            return None 

     def sendtointr(self, ev):
         menuitem = ev.getSource()
         if self.bytes_ != None and self.cache_file != False:
            print("[+] Starting write to file")
            try:
               # self.cache_file.write(self.bytes_)
               for byte in self.bytes_:
                   print >> self.cache_file, byte
            except IOError as e:
                print("[-] Write failure: " + e)
            print("[+] Write to file complete!")
         
            
        
         

