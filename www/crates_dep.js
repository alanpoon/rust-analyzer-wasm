import JSZipUtils from 'jszip-utils';
import untar from 'js-untar';
import Pako from 'pako';
export default class WishesStore{
    constructor(params)
    {
        this.name="";
        this.db=null;
        this.version=null;
    }
    trace (msg) {
        //Traces
        console.log(msg);
    }
    init(dbname, dbversion){
        var __this = this;
        //1.Initialize variables
        __this.name = dbname;
        __this.version = dbversion;
        //2. Make indexedDB compatible
        console.log("this.compatibility()",__this.compatibility());
        if (__this.compatibility()) {

            //2.1 Delete database
            //deletedb("wishes");
            //3.Open database
            __this.open();
        }
    }
    compatibility(){
        var trace = this.trace;
        trace("window.indexedDB: " + window.indexedDB);
        trace("window.mozIndexedDB: " + window.mozIndexedDB);
        trace("window.webkitIndexedDB: " + window.webkitIndexedDB);
        trace("window.msIndexedDB: " + window.msIndexedDB);

        //window.indexedDB = window.indexedDB || window.webkitIndexedDB || window.mozIndexedDB;

        trace("window.IDBTransaction: " + window.IDBTransaction);
        trace("window.webkitIDBTransaction: " + window.webkitIDBTransaction);
        trace("window.msIDBTransaction: " + window.msIDBTransaction);

        window.IDBTransaction = window.IDBTransaction || window.webkitIDBTransaction || window.msIDBTransaction || window.OIDBTransaction;

        trace("window.IDBKeyRange: " + window.IDBKeyRange);
        //trace("window.webkitIDBKeyRange: " + window.webkitIDBKeyRange);
        trace("window.msIDBKeyRange: " + window.msIDBKeyRange);

        window.IDBKeyRange = window.IDBKeyRange || window.webkitIDBKeyRange || window.msIDBKeyRange;

        if (window.indexedDB) {
            // var span = document.querySelector("header h1 span");
            // span.textContent = "Yes";
            // span.style.color = "green";
            return true;
        }

        trace("Your browser does not support a stable version of IndexedDB.");
        return false;

    }
    deletedb(dbname) {
        var trace = this.trace;
        trace("Delete " + dbname);

        var request = window.indexedDB.deleteDatabase(dbname);

        request.onsuccess = function() {
            trace("Database " + dbname + " deleted!");
        };

        request.onerror = function(event) {
            trace("deletedb(); error: " + event);
        };
    }
    open() {
        var __this = this;

        //3.1. Open a database async
        var request = window.indexedDB.open("wishes", 1);

        //3.2 The database has changed its version (For IE 10 and Firefox)
        request.onupgradeneeded = function(event) {

            __this.trace("Upgrade needed!");

            __this.db = event.target.result;

            __this.modifydb(); //Here we can modify the database
        };

        request.onsuccess = function(event) {
            __this.trace("Database opened");

            __this.db = event.target.result;
            console.log("__this.version",__this.version," __this.db.version", __this.db.version,event.target.result)
            //3.2 The database has changed its version (For Chrome)
            if (__this.version != __this.db.version && window.webkitIndexedDB) {

                __this.trace("version is different");

                var setVersionreq = __this.db.setVersion(__this.version);

                setVersionreq.onsuccess = __this.modifydb; //Here we can modify the database
            }
            console.log("Let's paint..")
            
            __this.trace("Let's paint");
            __this.items(); //4. Read our previous objects in the store (If there are any)
        };

        request.onerror = function(event) {
            __this.trace("Database error: " + event);
        };
    }
    modifydb() {
        //3.3 Create / Modify object stores in our database 
        //2.Delete previous object store
        if (this.db.objectStoreNames.contains("mywishes")) {
            this.db.deleteObjectStore("mywishes");
            this.trace("db.deleteObjectStore('mywishes');");
        }

        //3.Create object store
        var store = this.db.createObjectStore("mywishes", {
            keyPath: "file_name"
        });
        console.log("modifydb..")
    }
    add() {
        var __this =this;
        //4. Add objects
        __this.trace("add();");

        var trans = __this.db.transaction(["mywishes"], "readwrite"),
            store = trans.objectStore("mywishes"),
            wish = document.getElementById("wish").value;

        var data = {
            text: wish,
            //"timeStamp": new Date().getTime(),
            "file_name":file_name,
        };

        var request = store.add(data);

        request.onsuccess = function(event) {
            //this.trace("wish added!");
            __this.items(); //5 Read items after adding
        };
    }
    add_crate(crate_link,crate_name) {
        //4. Add objects
        var __this = this;
        var trace = __this.trace;
        console.log("add_crate();",crate_link);
        JSZipUtils.getBinaryContent(crate_link, function(err, data) {
            if(err) {
                trace("jsziputillis",err);
                throw err; // or handle err
            }
            var res = Pako.inflate(data);
            trace("res.buffer",data);
            untar(res.buffer).then(
            function(extractedFiles) { // onSuccess
                trace("extractedFiles",extractedFiles);
                var aggr_lib_text = "";
                
                var data = {
                    "anchored":{

                    },
                    "Cargo.toml":"",
                    "file_name":crate_name,
                };
                //document.getElementById('results').textContent = JSON.stringify(extractedFiles, null, 2);
                for (var i=0;i<extractedFiles.length;i++){
                    var file_name = extractedFiles[i].name;
                    if (file_name.includes(".rs") && !file_name.includes("build.rs")) {
                        var p = file_name.split("/src/");
                        if (p.length>1){
                            var rel_path = "/" + crate_name + "/src/"+p[1];
                            var text = extractedFiles[i].readAsString();
                            data["anchored"][rel_path] = text;
                        }

                    }else if (file_name.includes("Cargo.toml")){
                        var text = extractedFiles[i].readAsString();
                        data["Cargo.toml"] = text;
                    }
                }
                    
                var trans = __this.db.transaction(["mywishes"], "readwrite"),
                store = trans.objectStore("mywishes");
                //var request = store.add(data,file_name);
                
                var request = store.add(data);
                request.onsuccess = function(event) {
                    //__this.trace("wish added!",file_name);
                    __this.items(); //5 Read items after adding
                };
            },
            function(err) { // onError
                //... // Handle the error.
                console.log("err",err)
            }
            );
        });
    
        
    }
    items() {
        //5. Read
        var __this = this;
        //__this.trace("items(); called");
        var trans = this.db.transaction(["mywishes"], "readonly"),
            store = trans.objectStore("mywishes");

        var keyRange = IDBKeyRange.lowerBound(0);
        var cursorRequest = store.openCursor(keyRange);

        cursorRequest.onsuccess = function(event) {
            //__this.trace("Cursor opened!");

            var result = event.target.result;

            if (result === false || result === null){
                return;
            }
            __this.render(result.value.file_name); //4.1 Create HTML elements for this object
            result.continue ();

        };
    }
    items_list(){
        var trace = this.trace;
        var list = document.getElementById("list"),
            trans = this.db.transaction(["mywishes"], "readonly"),
            store = trans.objectStore("mywishes");
        var keyRange = IDBKeyRange.lowerBound(0);
        var cursorRequest = store.openCursor(keyRange);
        var crates_in_db=[];
        cursorRequest.onsuccess = function(event) {
            //trace("Cursor opened!");

            var result = event.target.result;

            if (result === false || result === null){
                return;
            }
            var crate = result.value.file_name.split("/")[0].split("-")[0];
            if (!crates_in_db.includes(crate)){
                crates_in_db.push(crate);
            }
            console.log("v",result)
        //    render(result.value.file_name); //4.1 Create HTML elements for this object
            result.continue ();

        };
        return crates_in_db;
    }
    render(item) {
        var __this = this;
        //5.1 Create DOM elements
        //__this.trace("Render items");

        // var list = document.getElementById("list"),
        //     li = document.createElement("li"),
        //     a = document.createElement("a"),
        //     text = document.createTextNode(item);

        // a.textContent = " X";
        // li.appendChild(text);
        // li.appendChild(a);
        // list.appendChild(li);
    }
    del(file_name) {
        var __this = this;
        //6. Delete items
        var transaction = this.db.transaction(["mywishes"], "readwrite");
        var store = transaction.objectStore("mywishes");

        var request = store.delete(file_name);

        request.onsuccess = function(event) {
            __this.trace("Item deleted!");
            __this.items(); //5.1 Read items after deleting
        };

        request.onerror = function(event) {
            trace("Error deleting: " + e);
        };
    }
}