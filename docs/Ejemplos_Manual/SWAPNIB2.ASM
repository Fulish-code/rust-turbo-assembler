;**********************************************************
; SWAPNIB2.ASM
;
;** En el manual se llama SWAPNIBL2.ASM; DOS sólo admite nombres
;** de 8 caracteres, por eso aquí se llama SWAPNIB2.ASM.
;
; Este programa intercambia los MSN y LSN de una variable
; de un byte. El resultado queda en la misma variable.
; Este programa utiliza un procedimiento para intercambiar
; los nibles.
;
; El pseudocódigo de este programa es:
;
;    AL = dato
;    swapnibl()
;    dato = AL
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
dato    db      ?

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     al, [dato]              ; AL = dato

; Llama al procedimiento para intercambiar los nibles
        call    swapnibl                ; swapnibl()

        mov     [dato], al              ; dato = AL

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** PROCEDIMIENTOS ************************************

;**********************************************************
; SWAPNIBL
;
; Este procedimiento intercambia los MSN y LSN de una
; variable de un byte.
;
; Parámetro:
;
;       AL = dato
;
; Regresa:
;
;       AL = dato con los nibles intercambiados.
;
; El pseudocódigo de este procedimiento es:
;
;     CL = 4
;     rol al, 4
;**********************************************************
proc    swapnibl
        push    cx                      ; Preserva CX

        mov     cl, 4
        rol     al, cl

        pop     cx                      ; Recupera CX
        ret
endp    swapnibl

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
