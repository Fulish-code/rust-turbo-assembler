;**********************************************************
; HOLA.ASM
;
; Primer programa: despliega "Hola mundo" en la pantalla con
; la función 09h de la interrupción 21h del DOS (despliega
; una cadena terminada en '$').
;
; Pruébalo:
;   - Ejecutar (Ctrl+F9): ensambla, enlaza y muestra el saludo.
;   - Depurar (F8): abre Turbo Debugger. Avanza con F8 y usa
;     Alt+F5 para ver la pantalla del programa.
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
saludo  db      'Hola mundo', 13, 10, '$'   ; 13, 10 = salto de línea

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     ah, 09h                 ; Despliega la cadena
        mov     dx, offset saludo       ; DS:DX = saludo
        int     21h

salir:
        mov     ah, 04Ch                ; Termina el programa
        mov     al, [codsal]            ; AL = código de salida
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
